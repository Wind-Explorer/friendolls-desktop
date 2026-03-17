// in app-core/src/state.rs
use crate::{
    lock_w,
    models::app_data::UserData,
    services::{
        app_data::update_display_dimensions_for_scene_state,
        presence_modules::models::ModuleMetadata,
    },
};
use std::sync::{Arc, LazyLock, RwLock};
use tauri::tray::TrayIcon;
use tracing::info;

pub mod auth;
mod network;

pub use auth::*;
pub use network::*;

#[derive(Default)]
pub struct Modules {
    pub handles: std::sync::Mutex<Vec<std::thread::JoinHandle<()>>>,
    pub metadatas: Vec<ModuleMetadata>,
}

#[derive(Default)]
pub struct AppState {
    pub app_config: crate::services::client_config::AppConfig,
    pub network: NetworkState,
    pub auth: AuthState,
    pub user_data: UserData,
    pub tray: Option<TrayIcon>,
    pub modules: Modules,
}

// Global application state
// Read / write this state via the `lock_r!` / `lock_w!` macros from `fdoll-core::utilities`
pub static FDOLL: LazyLock<Arc<RwLock<AppState>>> =
    LazyLock::new(|| Arc::new(RwLock::new(AppState::default())));

/// Populate app state with initial
/// values and necesary client instances.
pub fn init_app_state() {
    dotenvy::dotenv().ok();
    {
        let mut guard = lock_w!(FDOLL);
        guard.app_config = crate::services::client_config::load_app_config();
        guard.network = init_network_state();
        guard.auth = init_auth_state();
        guard.user_data = UserData::default();
        guard.modules = Modules::default();
    }
    update_display_dimensions_for_scene_state();
    info!("Initialized FDOLL state (WebSocket client & user data initializing asynchronously)");
}
