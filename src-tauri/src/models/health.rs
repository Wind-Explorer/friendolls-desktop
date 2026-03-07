use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

#[derive(Default, Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponseDto {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub db: String,
}

#[derive(Error, Debug)]
pub enum HealthError {
    #[error("app configuration missing {0}")]
    ConfigMissing(&'static str),
    #[error("health request failed: {0}")]
    Request(reqwest::Error),
    #[error("unexpected health status: {0}")]
    UnexpectedStatus(StatusCode),
    #[error("health status reported not OK: {0}")]
    NonOkStatus(String),
    #[error("health response decode failed: {0}")]
    Decode(reqwest::Error),
}
