use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const ICON_SIZE: u32 = 64;
pub const ICON_CACHE_LIMIT: usize = 50;

/// Metadata for the currently active application, including localized and unlocalized names, and an optional base64-encoded icon.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppMetadata {
    pub localized: Option<String>,
    pub unlocalized: Option<String>,
    pub app_icon_b64: Option<String>,
}
