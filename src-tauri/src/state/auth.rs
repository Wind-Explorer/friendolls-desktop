use crate::services::auth::{load_auth_pass, AuthPass};
use tracing::{info, warn};

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
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            auth_pass: None,
            oauth_flow: OAuthFlowTracker::default(),
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
    }
}