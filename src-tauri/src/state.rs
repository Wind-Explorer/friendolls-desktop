// in app-core/src/state.rs
use crate::{
    get_app_handle, lock_w,
    models::{
        app_config::{AppConfig, AuthConfig},
        app_data::AppData,
    },
    remotes::{friends::FriendRemote, user::UserRemote},
    services::auth::{load_auth_pass, AuthPass},
};
use serde_json::json;
use std::{
    env,
    sync::{Arc, LazyLock, RwLock},
};
use tauri::{async_runtime, Emitter};
use tracing::{info, warn};

#[derive(Default, Clone)]
pub struct OAuthFlowTracker {
    pub state: Option<String>,
    pub code_verifier: Option<String>,
    pub initiated_at: Option<u64>,
}

pub struct Clients {
    pub http_client: reqwest::Client,
    pub ws_client: Option<rust_socketio::client::Client>,
}

#[derive(Default)]
pub struct AppState {
    pub app_config: AppConfig,
    pub clients: Option<Clients>,
    pub auth_pass: Option<AuthPass>,
    pub oauth_flow: OAuthFlowTracker,

    // exposed to the frontend
    pub app_data: AppData,
}

// Global application state
// Read / write this state via the `lock_r!` / `lock_w!` macros from `fdoll-core::utilities`
pub static FDOLL: LazyLock<Arc<RwLock<AppState>>> =
    LazyLock::new(|| Arc::new(RwLock::new(AppState::default())));

pub fn init_fdoll_state() {
    {
        let mut guard = lock_w!(FDOLL);
        dotenvy::dotenv().ok();
        guard.app_config = AppConfig {
            api_base_url: Some(env::var("API_BASE_URL").expect("API_BASE_URL must be set")),
            auth: AuthConfig {
                audience: env::var("JWT_AUDIENCE").expect("JWT_AUDIENCE must be set"),
                auth_url: env::var("AUTH_URL").expect("AUTH_URL must be set"),
                redirect_uri: env::var("REDIRECT_URI").expect("REDIRECT_URI must be set"),
                redirect_host: env::var("REDIRECT_HOST").expect("REDIRECT_HOST must be set"),
            },
        };
        guard.auth_pass = match load_auth_pass() {
            Ok(pass) => pass,
            Err(e) => {
                warn!("Failed to load auth pass from keyring: {e}");
                None
            }
        };
        info!("Loaded auth pass");

        // Initialize HTTP client immediately (non-blocking)
        let http_client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .user_agent("friendolls-desktop/0.1.0")
            .build()
            .expect("Client should build");

        // Store HTTP client immediately - WebSocket client will be added later
        guard.clients = Some(Clients {
            http_client,
            ws_client: None,
        });
        info!("Initialized HTTP client");

        let has_auth = guard.auth_pass.is_some();

        drop(guard);

        if has_auth {
            async_runtime::spawn(async move {
                crate::services::ws::init_ws_client().await;
            });
        }

        info!("Initialized FDOLL state (WebSocket client & user data initializing asynchronously)");
    }
}

/// To be called in init state or need to refresh data.
/// Populate user data in app state from the server.
pub async fn init_app_data() {
    let user_remote = UserRemote::new();
    let friend_remote = FriendRemote::new();
    let user = user_remote
        .get_user(None)
        .await
        .expect("TODO: handle user profile fetch failure");
    let friends = friend_remote
        .get_friends()
        .await
        .expect("TODO: handle friends fetch failure");

    {
        let mut guard = lock_w!(FDOLL);
        guard.app_data.user = Some(user);
        guard.app_data.friends = Some(friends);
        get_app_handle()
            .emit("app-data-refreshed", json!(guard.app_data))
            .expect("TODO: handle event emit fail");
    }
}
