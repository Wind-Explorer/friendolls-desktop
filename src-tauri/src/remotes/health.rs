use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
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

pub struct HealthRemote {
    pub base_url: String,
    pub client: Client,
}

impl HealthRemote {
    pub fn new() -> Self {
        let guard = lock_r!(FDOLL);
        Self {
            base_url: guard
                .app_config
                .api_base_url
                .as_ref()
                .expect("App configuration error")
                .clone(),
            client: guard
                .clients
                .as_ref()
                .expect("App configuration error")
                .http_client
                .clone(),
        }
    }

    pub async fn get_health(&self) -> Result<HealthResponseDto, Error> {
        let url = format!("{}/health", self.base_url);
        let resp = self.client.get(url).send().await?;
        let health = resp.json().await?;
        Ok(health)
    }
}
