// in app-core/src/state.rs
use crate::{core::models::app_config::AppConfig, lock_w};
use reqwest::Client;
use std::sync::{Arc, LazyLock, RwLock};

#[derive(Default)]
pub struct AppState {
    pub app_config: Option<AppConfig>,
    pub http_client: Client,
}

// Global application state
// FDOLL = Multiplayer Todo App
// Read / write this state via the `lock_r!` / `lock_w!` macros from `fdoll-core::utilities`
pub static FDOLL: LazyLock<Arc<RwLock<AppState>>> =
    LazyLock::new(|| Arc::new(RwLock::new(AppState::default())));

pub fn init_fdoll_state() {
    {
        let mut guard = lock_w!(FDOLL);
        guard.app_config = Some(AppConfig {
            api_base_url: Some("http://sandbox:3000".to_string()),
        });
        guard.http_client = reqwest::Client::new();
    }
}
