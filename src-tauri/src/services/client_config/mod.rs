mod store;
mod window;

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;
use thiserror::Error;

pub use store::{load_app_config, save_app_config};
pub use window::open_config_window;

pub use crate::services::accelerators::{
    default_accelerator_for_action, default_accelerators, normalize_accelerators,
    AcceleratorAction, KeyboardAccelerator,
};

#[derive(Serialize, Clone, Debug, Type)]
pub struct AppConfig {
    pub api_base_url: Option<String>,
    pub debug_mode: bool,
    #[serde(default)]
    pub accelerators: BTreeMap<AcceleratorAction, KeyboardAccelerator>,
}

impl AppConfig {
    pub fn normalized(mut self) -> Self {
        self.accelerators = normalize_accelerators(self.accelerators);
        self
    }

    pub fn accelerator_for(&self, action: AcceleratorAction) -> KeyboardAccelerator {
        self.accelerators
            .get(&action)
            .cloned()
            .unwrap_or_else(|| default_accelerator_for_action(action))
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_base_url: None,
            debug_mode: false,
            accelerators: default_accelerators(),
        }
    }
}

impl<'de> Deserialize<'de> for AppConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct AppConfigSerde {
            api_base_url: Option<String>,
            #[serde(default)]
            debug_mode: bool,
            #[serde(default)]
            accelerators: BTreeMap<AcceleratorAction, KeyboardAccelerator>,
        }

        let value = AppConfigSerde::deserialize(deserializer)?;

        Ok(Self {
            api_base_url: value.api_base_url,
            debug_mode: value.debug_mode,
            accelerators: value.accelerators,
        }
        .normalized())
    }
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
    #[error("missing required parent window: {0}")]
    MissingParent(String),
}

pub static CLIENT_CONFIG_WINDOW_LABEL: &str = "client_config";
