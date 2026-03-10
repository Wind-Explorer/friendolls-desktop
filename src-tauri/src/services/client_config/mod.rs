mod store;
mod window;

use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

pub use store::{load_app_config, save_app_config};
pub use window::open_config_window;

#[derive(Default, Serialize, Deserialize, Clone, Debug, Type)]
pub struct AppConfig {
    pub api_base_url: Option<String>,
}

#[derive(Debug, Error)]
pub enum ClientConfigError {
    #[error("failed to resolve app config dir: {0}")]
    ResolvePath(tauri::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse client config: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("failed to run on main thread: {0}")]
    Dispatch(#[from] tauri::Error),
    #[error("failed to build client config window: {0}")]
    Window(tauri::Error),
    #[error("failed to show client config window: {0}")]
    ShowWindow(tauri::Error),
}

pub static CLIENT_CONFIG_WINDOW_LABEL: &str = "client_config";
