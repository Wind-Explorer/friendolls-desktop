use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::Manager;
use thiserror::Error;
use tracing::{error, warn};
use url::Url;

use crate::get_app_handle;

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct AuthConfig {
    pub audience: String,
    pub auth_url: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub api_base_url: Option<String>,
    pub auth: AuthConfig,
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
    #[error("failed to build client config manager window: {0}")]
    Window(tauri::Error),
    #[error("failed to show client config manager window: {0}")]
    ShowWindow(tauri::Error),
}

pub static CLIENT_CONFIG_MANAGER_WINDOW_LABEL: &str = "client_config_manager";
const CONFIG_FILENAME: &str = "client_config.json";
const DEFAULT_API_BASE_URL: &str = "https://api.fdolls.adamcv.com";
const DEFAULT_AUTH_URL: &str = "https://auth.adamcv.com/realms/friendolls/protocol/openid-connect";
const DEFAULT_JWT_AUDIENCE: &str = "friendolls-desktop";

fn config_file_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, ClientConfigError> {
    let dir = app_handle
        .path()
        .app_config_dir()
        .map_err(ClientConfigError::ResolvePath)?;
    Ok(dir.join(CONFIG_FILENAME))
}

fn strip_trailing_slash(value: &str) -> String {
    value.trim_end_matches('/').to_string()
}

fn parse_http_url(value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }

    let attempts = [value.to_string(), format!("https://{value}")];

    for attempt in attempts {
        if let Ok(parsed) = Url::parse(&attempt) {
            if matches!(parsed.scheme(), "http" | "https") {
                return Some(strip_trailing_slash(parsed.as_str()));
            }
        }
    }

    None
}

fn sanitize(mut config: AppConfig) -> AppConfig {
    config.api_base_url = config
        .api_base_url
        .and_then(|v| parse_http_url(v.trim()))
        .or_else(|| Some(DEFAULT_API_BASE_URL.to_string()))
        .map(|v| strip_trailing_slash(&v));

    let auth_url_trimmed = config.auth.auth_url.trim();
    config.auth.auth_url =
        parse_http_url(auth_url_trimmed).unwrap_or_else(|| DEFAULT_AUTH_URL.to_string());

    if config.auth.audience.trim().is_empty() {
        config.auth.audience = DEFAULT_JWT_AUDIENCE.to_string();
    } else {
        config.auth.audience = config.auth.audience.trim().to_string();
    }

    config
}

pub fn default_app_config() -> AppConfig {
    AppConfig {
        api_base_url: Some(DEFAULT_API_BASE_URL.to_string()),
        auth: AuthConfig {
            audience: DEFAULT_JWT_AUDIENCE.to_string(),
            auth_url: DEFAULT_AUTH_URL.to_string(),
        },
    }
}

pub fn load_app_config() -> AppConfig {
    let app_handle = get_app_handle();
    let path = match config_file_path(app_handle) {
        Ok(p) => p,
        Err(e) => {
            warn!("Unable to resolve client config path: {e}");
            return default_app_config();
        }
    };

    if !path.exists() {
        return default_app_config();
    }

    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
            Ok(cfg) => sanitize(cfg),
            Err(e) => {
                warn!("Failed to parse client config, using defaults: {e}");
                default_app_config()
            }
        },
        Err(e) => {
            warn!("Failed to read client config, using defaults: {e}");
            default_app_config()
        }
    }
}

pub fn save_app_config(config: AppConfig) -> Result<AppConfig, ClientConfigError> {
    let app_handle = get_app_handle();
    let path = config_file_path(app_handle)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let sanitized = sanitize(config);
    let serialized = serde_json::to_string_pretty(&sanitized)?;

    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, serialized)?;
    fs::rename(&temp_path, &path)?;

    Ok(sanitized)
}

pub fn open_config_manager_window() -> Result<(), ClientConfigError> {
    let app_handle = get_app_handle();

    // Directly run on main thread via dispatch but handle errors properly
    // This is essentially what we did but let's simplify and make sure we don't block
    let handle_for_closure = app_handle.clone();

    // We want to return immediately, the window creation happens asynchronously on main thread
    let _ = app_handle.run_on_main_thread(move || {
        if let Err(e) = open_config_manager_window_inner(&handle_for_closure) {
            error!("Failed to open client config manager window: {e}");
        }
    });

    Ok(())
}

fn open_config_manager_window_inner(
    app_handle: &tauri::AppHandle,
) -> Result<(), ClientConfigError> {
    let existing_webview_window = app_handle.get_window(CLIENT_CONFIG_MANAGER_WINDOW_LABEL);

    if let Some(window) = existing_webview_window {
        if let Err(e) = window.show() {
            error!("Failed to show client config manager window: {e}");
            return Err(ClientConfigError::ShowWindow(e));
        }
        if let Err(e) = window.set_focus() {
            error!("Failed to focus client config manager window: {e}");
        }
        return Ok(());
    }

    match tauri::WebviewWindowBuilder::new(
        app_handle,
        CLIENT_CONFIG_MANAGER_WINDOW_LABEL,
        tauri::WebviewUrl::App("/client-config-manager".into()),
    )
    .title("Advanced Configuration")
    .inner_size(300.0, 420.0)
    .resizable(false)
    .maximizable(false)
    .visible(false)
    .build()
    {
        Ok(window) => {
            if let Err(e) = window.show() {
                error!("Failed to show client config manager window: {}", e);
                return Err(ClientConfigError::ShowWindow(e));
            }
            if let Err(e) = window.set_focus() {
                error!("Failed to focus client config manager window: {e}");
            }
            Ok(())
        }
        Err(e) => {
            error!("Failed to build client config manager window: {}", e);
            Err(ClientConfigError::Window(e))
        }
    }
}
