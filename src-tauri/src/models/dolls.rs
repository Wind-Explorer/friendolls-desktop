use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

#[derive(Error, Debug)]
pub enum RemoteError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DollColorSchemeDto {
    pub outline: String,
    pub body: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DollConfigurationDto {
    pub color_scheme: DollColorSchemeDto,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateDollDto {
    pub name: String,
    pub configuration: Option<DollConfigurationDto>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateDollDto {
    pub name: Option<String>,
    pub configuration: Option<DollConfigurationDto>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DollDto {
    pub id: String,
    pub name: String,
    pub configuration: DollConfigurationDto,
    pub created_at: String,
    pub updated_at: String,
}