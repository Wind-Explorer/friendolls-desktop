use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PresenceStatus {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub graphics_b64: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct ModuleMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}
