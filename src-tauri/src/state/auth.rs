use crate::services::auth::{clear_auth_pass, load_auth_pass, refresh_token, AuthPass};
use crate::services::{session::destruct_user_session, welcome::open_welcome_window};
use crate::{lock_r, lock_w, state::FDOLL};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

static REFRESH_LOCK: once_cell::sync::Lazy<Mutex<()>> =
    once_cell::sync::Lazy::new(|| Mutex::new(()));

#[derive(Default, Clone)]
pub struct OAuthFlowTracker {
    pub active_flow_id: u64,
    pub background_auth_token: Option<CancellationToken>,
}

#[derive(Default)]
pub struct AuthState {
    pub auth_pass: Option<AuthPass>,
    pub oauth_flow: OAuthFlowTracker,
    pub background_refresh_token: Option<CancellationToken>,
}

pub fn init_auth_state() -> AuthState {
    let auth_pass = match load_auth_pass() {
        Ok(Some(pass)) if has_supported_auth_pass(&pass) => Some(pass),
        Ok(Some(_)) => {
            warn!("Discarding stored auth pass from unsupported auth format");
            if let Err(err) = clear_auth_pass() {
                error!("Failed to clear unsupported auth pass: {}", err);
            }
            None
        }
        Ok(None) => None,
        Err(e) => {
            warn!("Failed to load auth pass from keyring: {e}");
            None
        }
    };
    info!("Loaded auth pass");

    AuthState {
        auth_pass,
        oauth_flow: OAuthFlowTracker::default(),
        background_refresh_token: None,
    }
}

fn has_supported_auth_pass(auth_pass: &AuthPass) -> bool {
    auth_pass.issued_at.is_some()
        && auth_pass.refresh_token.is_some()
        && auth_pass.refresh_expires_in.is_some()
}

pub fn begin_auth_flow() -> (u64, CancellationToken) {
    let mut guard = lock_w!(FDOLL);
    if let Some(cancel_token) = guard.auth.oauth_flow.background_auth_token.take() {
        cancel_token.cancel();
    }

    guard.auth.oauth_flow.active_flow_id = guard.auth.oauth_flow.active_flow_id.saturating_add(1);
    let flow_id = guard.auth.oauth_flow.active_flow_id;
    let cancel_token = CancellationToken::new();
    guard.auth.oauth_flow.background_auth_token = Some(cancel_token.clone());

    (flow_id, cancel_token)
}

pub fn clear_auth_flow_state(flow_id: u64) -> bool {
    let mut guard = lock_w!(FDOLL);
    if guard.auth.oauth_flow.active_flow_id != flow_id {
        return false;
    }

    if let Some(cancel_token) = guard.auth.oauth_flow.background_auth_token.take() {
        cancel_token.cancel();
    }

    true
}

pub fn is_auth_flow_active(flow_id: u64) -> bool {
    let guard = lock_r!(FDOLL);
    guard.auth.oauth_flow.active_flow_id == flow_id
        && guard.auth.oauth_flow.background_auth_token.is_some()
}

/// Returns the auth pass object, including access token and metadata.
/// Automatically refreshes if expired and clears session on refresh failure.
pub async fn get_auth_pass_with_refresh() -> Option<AuthPass> {
    info!("Retrieving tokens");
    let auth_pass = lock_r!(FDOLL).auth.auth_pass.clone()?;

    let Some(issued_at) = auth_pass.issued_at else {
        warn!("Auth pass missing issued_at timestamp, clearing");
        clear_invalid_auth().await;
        return None;
    };

    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let access_expires_at = issued_at.saturating_add(auth_pass.expires_in);
    let refresh_expires_at = auth_pass
        .refresh_expires_in
        .map(|refresh_expires_in| issued_at.saturating_add(refresh_expires_in));

    if refresh_expires_at.is_some_and(|refresh_expires_at| current_time >= refresh_expires_at) {
        clear_expired_auth().await;
        return None;
    }

    if current_time < access_expires_at {
        return Some(auth_pass);
    }

    let _guard = REFRESH_LOCK.lock().await;

    let auth_pass = lock_r!(FDOLL).auth.auth_pass.clone()?;
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let Some(issued_at) = auth_pass.issued_at else {
        warn!("Auth pass missing issued_at timestamp after refresh lock, clearing");
        clear_invalid_auth().await;
        return None;
    };

    let access_expires_at = issued_at.saturating_add(auth_pass.expires_in);
    let refresh_expires_at = auth_pass
        .refresh_expires_in
        .map(|refresh_expires_in| issued_at.saturating_add(refresh_expires_in));

    if refresh_expires_at.is_some_and(|refresh_expires_at| current_time >= refresh_expires_at) {
        clear_expired_auth().await;
        return None;
    }

    if current_time < access_expires_at {
        return Some(auth_pass);
    }

    info!("Access token expired, attempting refresh");
    let Some(refresh_token_value) = auth_pass.refresh_token.as_deref() else {
        warn!("Auth pass missing refresh token, clearing session");
        clear_invalid_auth().await;
        return None;
    };

    match refresh_token(refresh_token_value).await {
        Ok(new_pass) => Some(new_pass),
        Err(e) => {
            error!("Failed to refresh token: {}", e);
            clear_expired_auth().await;
            None
        }
    }
}

async fn clear_expired_auth() {
    lock_w!(FDOLL).auth.auth_pass = None;
    if let Err(e) = clear_auth_pass() {
        error!("Failed to clear expired auth pass: {}", e);
    }
    destruct_user_session().await;
    open_welcome_window();
}

async fn clear_invalid_auth() {
    clear_expired_auth().await;
}

async fn refresh_if_expiring_soon() {
    let Some(auth_pass) = ({ lock_r!(FDOLL).auth.auth_pass.clone() }) else {
        return;
    };

    let Some(issued_at) = auth_pass.issued_at else {
        return;
    };

    let current_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(value) => value.as_secs(),
        Err(_) => return,
    };

    let access_expires_at = issued_at.saturating_add(auth_pass.expires_in);
    if current_time >= access_expires_at && auth_pass.refresh_token.is_none() {
        clear_expired_auth().await;
        return;
    }

    let refresh_expires_at = auth_pass
        .refresh_expires_in
        .map(|refresh_expires_in| issued_at.saturating_add(refresh_expires_in));
    if refresh_expires_at.is_some_and(|refresh_expires_at| current_time >= refresh_expires_at) {
        clear_expired_auth().await;
        return;
    }

    if access_expires_at.saturating_sub(current_time) >= 60 {
        return;
    }

    let _guard = REFRESH_LOCK.lock().await;

    let Some(latest_pass) = ({ lock_r!(FDOLL).auth.auth_pass.clone() }) else {
        return;
    };

    let Some(latest_issued_at) = latest_pass.issued_at else {
        return;
    };

    let current_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(value) => value.as_secs(),
        Err(_) => return,
    };

    let access_expires_at = latest_issued_at.saturating_add(latest_pass.expires_in);
    if current_time >= access_expires_at && latest_pass.refresh_token.is_none() {
        clear_expired_auth().await;
        return;
    }

    let refresh_expires_at = latest_pass
        .refresh_expires_in
        .map(|refresh_expires_in| latest_issued_at.saturating_add(refresh_expires_in));
    if refresh_expires_at.is_some_and(|refresh_expires_at| current_time >= refresh_expires_at) {
        clear_expired_auth().await;
        return;
    }

    if access_expires_at.saturating_sub(current_time) >= 60 {
        return;
    }

    let Some(refresh_token_value) = latest_pass.refresh_token.as_deref() else {
        return;
    };

    if let Err(e) = refresh_token(refresh_token_value).await {
        warn!("Background refresh failed: {}", e);
        clear_expired_auth().await;
    }
}

/// Starts a background loop to periodically refresh tokens when authenticated.
pub async fn start_background_token_refresh() {
    stop_background_token_refresh();
    let cancel_token = CancellationToken::new();
    {
        let mut guard = lock_w!(FDOLL);
        guard.auth.background_refresh_token = Some(cancel_token.clone());
    }
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60));
        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    break;
                }
                _ = interval.tick() => {
                    refresh_if_expiring_soon().await;
                }
            }
        }
    });
}

/// Stops the background token refresh loop.
pub fn stop_background_token_refresh() {
    if let Some(token) = lock_w!(FDOLL).auth.background_refresh_token.take() {
        token.cancel();
    }
}
