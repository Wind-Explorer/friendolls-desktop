use crate::init::lifecycle::destruct_user_session;
use crate::services::auth::{clear_auth_pass, load_auth_pass, refresh_token, AuthPass};
use crate::services::welcome::open_welcome_window;
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
    pub state: Option<String>,
    pub code_verifier: Option<String>,
    pub initiated_at: Option<u64>,
    pub cancel_token: Option<tokio_util::sync::CancellationToken>,
}

pub struct AuthState {
    pub auth_pass: Option<AuthPass>,
    pub oauth_flow: OAuthFlowTracker,
    pub background_refresh_token: Option<tokio_util::sync::CancellationToken>,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            auth_pass: None,
            oauth_flow: OAuthFlowTracker::default(),
            background_refresh_token: None,
        }
    }
}

pub fn init_auth_state() -> AuthState {
    let auth_pass = match load_auth_pass() {
        Ok(pass) => pass,
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

/// Returns the auth pass object, including access token, refresh token, and metadata.
/// Automatically refreshes if expired and clears session if refresh token is expired.
pub async fn get_auth_pass_with_refresh() -> Option<AuthPass> {
    info!("Retrieving tokens");
    let Some(auth_pass) = ({ lock_r!(FDOLL).auth.auth_pass.clone() }) else {
        return None;
    };

    let Some(issued_at) = auth_pass.issued_at else {
        warn!("Auth pass missing issued_at timestamp, clearing");
        lock_w!(FDOLL).auth.auth_pass = None;
        return None;
    };

    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let expired = current_time - issued_at >= auth_pass.expires_in;
    let refresh_expired = current_time - issued_at >= auth_pass.refresh_expires_in;

    if !expired {
        return Some(auth_pass);
    }

    if refresh_expired {
        info!("Refresh token expired, clearing auth state");
        lock_w!(FDOLL).auth.auth_pass = None;
        if let Err(e) = clear_auth_pass() {
            error!("Failed to clear expired auth pass: {}", e);
        }
        destruct_user_session().await;
        open_welcome_window();
        return None;
    }

    let _guard = REFRESH_LOCK.lock().await;

    let auth_pass = lock_r!(FDOLL).auth.auth_pass.clone()?;
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let Some(issued_at) = auth_pass.issued_at else {
        warn!("Auth pass missing issued_at timestamp after refresh lock, clearing");
        lock_w!(FDOLL).auth.auth_pass = None;
        return None;
    };
    let expired = current_time - issued_at >= auth_pass.expires_in;
    let refresh_expired = current_time - issued_at >= auth_pass.refresh_expires_in;

    if refresh_expired {
        info!("Refresh token expired, clearing auth state after refresh lock");
        lock_w!(FDOLL).auth.auth_pass = None;
        if let Err(e) = clear_auth_pass() {
            error!("Failed to clear expired auth pass: {}", e);
        }
        destruct_user_session().await;
        open_welcome_window();
        return None;
    }

    if !expired {
        return Some(auth_pass);
    }

    info!("Access token expired, attempting refresh");
    match refresh_token(&auth_pass.refresh_token).await {
        Ok(new_pass) => Some(new_pass),
        Err(e) => {
            error!("Failed to refresh token: {}", e);
            lock_w!(FDOLL).auth.auth_pass = None;
            if let Err(e) = clear_auth_pass() {
                error!("Failed to clear auth pass after refresh failure: {}", e);
            }
            None
        }
    }
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

    let refresh_expires_at = issued_at.saturating_add(auth_pass.refresh_expires_in);
    if current_time >= refresh_expires_at {
        lock_w!(FDOLL).auth.auth_pass = None;
        if let Err(e) = clear_auth_pass() {
            error!("Failed to clear expired auth pass: {}", e);
        }
        destruct_user_session().await;
        open_welcome_window();
        return;
    }

    let access_expires_at = issued_at.saturating_add(auth_pass.expires_in);
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

    let refresh_expires_at = latest_issued_at.saturating_add(latest_pass.refresh_expires_in);
    if current_time >= refresh_expires_at {
        lock_w!(FDOLL).auth.auth_pass = None;
        if let Err(e) = clear_auth_pass() {
            error!("Failed to clear expired auth pass: {}", e);
        }
        destruct_user_session().await;
        open_welcome_window();
        return;
    }

    let access_expires_at = latest_issued_at.saturating_add(latest_pass.expires_in);
    if access_expires_at.saturating_sub(current_time) >= 60 {
        return;
    }

    if let Err(e) = refresh_token(&latest_pass.refresh_token).await {
        warn!("Background refresh failed: {}", e);
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
