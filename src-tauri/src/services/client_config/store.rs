use std::{fs, path::PathBuf};

use tauri::Manager;
use tracing::warn;
use url::Url;

use crate::get_app_handle;

use super::{AppConfig, ClientConfigError};

const CONFIG_FILENAME: &str = "client_config.json";
const DEFAULT_API_BASE_URL: &str = "https://api.friendolls.adamcv.com";

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

    config
}

pub fn default_app_config() -> AppConfig {
    AppConfig {
        api_base_url: Some(DEFAULT_API_BASE_URL.to_string()),
        debug_mode: false,
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
