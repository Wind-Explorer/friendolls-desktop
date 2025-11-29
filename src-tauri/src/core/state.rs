// in app-core/src/state.rs
use crate::{
    core::{
        models::app_config::{AppConfig, AuthConfig},
        services::{
            auth::{load_auth_pass, AuthPass},
            ws::build_ws_client,
        },
    },
    lock_w,
};
use std::{
    env,
    sync::{Arc, LazyLock, RwLock},
};
use tauri::async_runtime;
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

        // Clone app_config for async task
        let app_config = guard.app_config.clone();

        // Drop the write lock before spawning async task
        drop(guard);

        // Initialize WebSocket client in a blocking task to avoid runtime conflicts
        async_runtime::spawn(async move {
            let ws_client = async_runtime::spawn_blocking(move || build_ws_client(&app_config))
                .await
                .expect("Failed to initialize WebSocket client");

            let mut guard = lock_w!(FDOLL);
            if let Some(clients) = guard.clients.as_mut() {
                clients.ws_client = Some(ws_client);
            }
            info!("Initialized FDOLL state with WebSocket client");
        });

        info!("Initialized FDOLL state (WebSocket client initializing asynchronously)");
    }
}
