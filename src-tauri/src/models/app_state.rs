use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
pub enum NekoPosition {
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SceneSetup {
    pub nekos_position: Option<NekoPosition>,
    pub nekos_opacity: f32,
    pub nekos_scale: f32,
}

impl Default for SceneSetup {
    fn default() -> Self {
        Self {
            nekos_position: None,
            nekos_opacity: 1.0,
            nekos_scale: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub scene_setup: SceneSetup,
}
