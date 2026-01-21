use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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