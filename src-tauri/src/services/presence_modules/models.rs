use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PresenceStatus {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub graphics_b64: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ModuleMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}
