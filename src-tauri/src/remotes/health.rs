use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{lock_r, state::FDOLL};

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

pub struct HealthRemote {
    pub base_url: String,
    pub client: Client,
}

impl HealthRemote {
    pub fn try_new() -> Result<Self, HealthError> {
        let guard = lock_r!(FDOLL);
        let base_url = guard
            .app_config
            .api_base_url
            .as_ref()
            .cloned()
            .ok_or(HealthError::ConfigMissing("api_base_url"))?;

        let client = guard
            .clients
            .as_ref()
            .map(|c| c.http_client.clone())
            .ok_or(HealthError::ConfigMissing("http_client"))?;

        Ok(Self { base_url, client })
    }

    pub async fn get_health(&self) -> Result<HealthResponseDto, HealthError> {
        let url = format!("{}/health", self.base_url);

        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(HealthError::Request)?;

        let resp = resp.error_for_status().map_err(|err| {
            err.status()
                .map(HealthError::UnexpectedStatus)
                .unwrap_or_else(|| HealthError::Request(err))
        })?;

        let health: HealthResponseDto = resp.json().await.map_err(HealthError::Decode)?;

        if health.status != "OK" {
            return Err(HealthError::NonOkStatus(health.status));
        }

        Ok(health)
    }
}
