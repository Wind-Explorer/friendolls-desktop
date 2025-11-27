// in app-core/src/state.rs
use crate::{
    core::models::app_config::{AppConfig, AuthConfig},
    core::services::auth::{load_auth_pass, AuthPass},
    lock_w,
};
use reqwest::Client;
use std::{
    env,
    sync::{Arc, LazyLock, RwLock},
};
use tracing::warn;

#[derive(Default, Clone)]
pub struct OAuthFlowTracker {
    pub state: Option<String>,
    pub code_verifier: Option<String>,
    pub initiated_at: Option<u64>,
}

#[derive(Default)]
pub struct AppState {
    pub app_config: Option<AppConfig>,
    pub http_client: Client,
    pub auth_pass: Option<AuthPass>,
    pub oauth_flow: OAuthFlowTracker,
}

// Global application state
// Read / write this state via the `lock_r!` / `lock_w!` macros from `fdoll-core::utilities`
pub static FDOLL: LazyLock<Arc<RwLock<AppState>>> =
    LazyLock::new(|| Arc::new(RwLock::new(AppState::default())));

pub fn init_fdoll_state() {
    {
        let mut guard = lock_w!(FDOLL);
        dotenvy::dotenv().ok();
        guard.app_config = Some(AppConfig {
            api_base_url: Some(env::var("API_BASE_URL").expect("API_BASE_URL must be set")),
            auth: AuthConfig {
                audience: env::var("JWT_AUDIENCE").expect("JWT_AUDIENCE must be set"),
                auth_url: env::var("AUTH_URL").expect("AUTH_URL must be set"),
                redirect_uri: env::var("REDIRECT_URI").expect("REDIRECT_URI must be set"),
                redirect_host: env::var("REDIRECT_HOST").expect("REDIRECT_HOST must be set"),
            },
        });
        guard.auth_pass = match load_auth_pass() {
            Ok(pass) => pass,
            Err(e) => {
                warn!("Failed to load auth pass from keyring: {e}");
                None
            }
        };
        guard.http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Client should build");
    }
}
