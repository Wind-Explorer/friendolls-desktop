mod store;
mod window;

use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;
use thiserror::Error;

pub use store::{load_app_config, save_app_config};
pub use window::open_config_window;

#[derive(Default, Serialize, Deserialize, Clone, Debug, Type)]
pub struct AppConfig {
    pub api_base_url: Option<String>,
    pub debug_mode: bool,
    #[serde(default)]
    pub scene_interactivity_hotkey: SceneInteractivityHotkey,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq, Type)]
pub struct SceneInteractivityHotkey {
    #[serde(default)]
    pub modifiers: Vec<SceneInteractivityModifier>,
    #[serde(default)]
    pub key: Option<SceneInteractivityKey>,
}

impl SceneInteractivityHotkey {
    pub fn normalized(mut self) -> Self {
        self.modifiers.sort_unstable();
        self.modifiers.dedup();

        if self.modifiers.is_empty() {
            return Self::default();
        }

        self
    }

    fn from_legacy(value: LegacySceneInteractivityHotkey) -> Self {
        let modifiers = match value {
            LegacySceneInteractivityHotkey::CmdAlt => {
                vec![
                    SceneInteractivityModifier::Cmd,
                    SceneInteractivityModifier::Alt,
                ]
            }
            LegacySceneInteractivityHotkey::CmdCtrl => {
                vec![
                    SceneInteractivityModifier::Cmd,
                    SceneInteractivityModifier::Ctrl,
                ]
            }
            LegacySceneInteractivityHotkey::AltCtrl => {
                vec![
                    SceneInteractivityModifier::Alt,
                    SceneInteractivityModifier::Ctrl,
                ]
            }
            LegacySceneInteractivityHotkey::Cmd => vec![SceneInteractivityModifier::Cmd],
            LegacySceneInteractivityHotkey::Alt => vec![SceneInteractivityModifier::Alt],
            LegacySceneInteractivityHotkey::Ctrl => vec![SceneInteractivityModifier::Ctrl],
        };

        Self {
            modifiers,
            key: None,
        }
    }
}

impl<'de> Deserialize<'de> for SceneInteractivityHotkey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SceneInteractivityHotkeyV2 {
            #[serde(default)]
            modifiers: Vec<SceneInteractivityModifier>,
            #[serde(default)]
            key: Option<SceneInteractivityKey>,
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum SceneInteractivityHotkeySerde {
            V2(SceneInteractivityHotkeyV2),
            Legacy(LegacySceneInteractivityHotkey),
        }

        let value = SceneInteractivityHotkeySerde::deserialize(deserializer)?;
        let normalized = match value {
            SceneInteractivityHotkeySerde::V2(v2) => Self {
                modifiers: v2.modifiers,
                key: v2.key,
            }
            .normalized(),
            SceneInteractivityHotkeySerde::Legacy(legacy) => Self::from_legacy(legacy).normalized(),
        };

        Ok(normalized)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Type)]
#[serde(rename_all = "snake_case")]
pub enum SceneInteractivityModifier {
    Cmd,
    Alt,
    Ctrl,
    Shift,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum SceneInteractivityKey {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Enter,
    Space,
    Escape,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Minus,
    Equal,
    LeftBracket,
    RightBracket,
    BackSlash,
    Semicolon,
    Apostrophe,
    Comma,
    Dot,
    Slash,
    Grave,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LegacySceneInteractivityHotkey {
    CmdAlt,
    CmdCtrl,
    AltCtrl,
    Cmd,
    Alt,
    Ctrl,
}

impl Default for SceneInteractivityHotkey {
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self {
                modifiers: vec![SceneInteractivityModifier::Cmd],
                key: None,
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Self {
                modifiers: vec![SceneInteractivityModifier::Alt],
                key: None,
            }
        }
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
