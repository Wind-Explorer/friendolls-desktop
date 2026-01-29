// in app-core/src/state.rs
use crate::lock_w;
use std::sync::{Arc, LazyLock, RwLock};
use tauri::tray::TrayIcon;
use tracing::info;

mod auth;
mod network;
mod ui;

pub use auth::*;
pub use network::*;
pub use ui::*;

#[derive(Default)]
pub struct AppState {
    pub app_config: crate::services::client_config_manager::AppConfig,
    pub network: NetworkState,
    pub auth: AuthState,
    pub ui: UiState,
    pub tray: Option<TrayIcon>,
}

// Global application state
// Read / write this state via the `lock_r!` / `lock_w!` macros from `fdoll-core::utilities`
pub static FDOLL: LazyLock<Arc<RwLock<AppState>>> =
    LazyLock::new(|| Arc::new(RwLock::new(AppState::default())));

pub fn init_fdoll_state() {
    {
        let mut guard = lock_w!(FDOLL);
        dotenvy::dotenv().ok();
        guard.app_config = crate::services::client_config_manager::load_app_config();
        guard.network = init_network_state();
        guard.auth = init_auth_state();
        guard.ui = init_ui_state();
    }

    info!("Initialized FDOLL state (WebSocket client & user data initializing asynchronously)");
}
