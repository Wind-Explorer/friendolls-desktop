use reqwest::Client;

use crate::{lock_r, services::auth::with_auth, state::FDOLL, models::dolls::*};
use crate::models::remote_error::RemoteError;

pub struct DollsRemote {
    pub base_url: String,
    pub client: Client,
}

impl DollsRemote {
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
                .network.clients
                .as_ref()
                .expect("App configuration error")
                .http_client
                .clone(),
        }
    }

    pub async fn get_dolls(&self) -> Result<Vec<DollDto>, RemoteError> {
        let url = format!("{}/dolls/me", self.base_url);
        tracing::info!(
            "DollsRemote::get_dolls - Sending GET request to URL: {}",
            url
        );
        let resp = with_auth(self.client.get(url)).await.send().await?;

        let resp = resp.error_for_status().map_err(|e| {
            tracing::error!("DollsRemote::get_dolls - HTTP error: {}", e);
            e
        })?;

        let text = resp.text().await.map_err(|e| {
            tracing::error!(
                "DollsRemote::get_dolls - Failed to read response text: {}",
                e
            );
            e
        })?;

        let dolls: Vec<DollDto> = serde_json::from_str(&text).map_err(|e| {
            tracing::error!("DollsRemote::get_dolls - Failed to parse JSON: {}", e);
            e
        })?;

        tracing::info!(
            "DollsRemote::get_dolls - Successfully parsed {} dolls",
            dolls.len()
        );
        Ok(dolls)
    }

    pub async fn get_doll(&self, id: &str) -> Result<DollDto, RemoteError> {
        let url = format!("{}/dolls/{}", self.base_url, id);
        tracing::info!(
            "DollsRemote::get_doll - Sending GET request to URL: {}",
            url
        );

        let resp = with_auth(self.client.get(url)).await.send().await?;

        let resp = resp.error_for_status().map_err(|e| {
            tracing::error!("DollsRemote::get_doll - HTTP error: {}", e);
            e
        })?;

        let text = resp.text().await.map_err(|e| {
            tracing::error!(
                "DollsRemote::get_doll - Failed to read response text: {}",
                e
            );
            e
        })?;

        let doll: DollDto = serde_json::from_str(&text).map_err(|e| {
            tracing::error!("DollsRemote::get_doll - Failed to parse JSON: {}", e);
            e
        })?;

        Ok(doll)
    }

    pub async fn create_doll(&self, dto: CreateDollDto) -> Result<DollDto, RemoteError> {
        let url = format!("{}/dolls", self.base_url);
        tracing::info!(
            "DollsRemote::create_doll - Sending POST request to URL: {}",
            url
        );

        let resp = with_auth(self.client.post(url))
            .await
            .json(&dto)
            .send()
            .await?;

        let resp = resp.error_for_status().map_err(|e| {
            tracing::error!("DollsRemote::create_doll - HTTP error: {}", e);
            e
        })?;

        let text = resp.text().await.map_err(|e| {
            tracing::error!(
                "DollsRemote::create_doll - Failed to read response text: {}",
                e
            );
            e
        })?;

        let doll: DollDto = serde_json::from_str(&text).map_err(|e| {
            tracing::error!("DollsRemote::create_doll - Failed to parse JSON: {}", e);
            e
        })?;

        Ok(doll)
    }

    pub async fn update_doll(&self, id: &str, dto: UpdateDollDto) -> Result<DollDto, RemoteError> {
        let url = format!("{}/dolls/{}", self.base_url, id);
        tracing::info!(
            "DollsRemote::update_doll - Sending PATCH request to URL: {}",
            url
        );

        let resp = with_auth(self.client.patch(url))
            .await
            .json(&dto)
            .send()
            .await?;

        let resp = resp.error_for_status().map_err(|e| {
            tracing::error!("DollsRemote::update_doll - HTTP error: {}", e);
            e
        })?;

        let text = resp.text().await.map_err(|e| {
            tracing::error!(
                "DollsRemote::update_doll - Failed to read response text: {}",
                e
            );
            e
        })?;

        let doll: DollDto = serde_json::from_str(&text).map_err(|e| {
            tracing::error!("DollsRemote::update_doll - Failed to parse JSON: {}", e);
            e
        })?;

        Ok(doll)
    }

    pub async fn delete_doll(&self, id: &str) -> Result<(), RemoteError> {
        let url = format!("{}/dolls/{}", self.base_url, id);
        tracing::info!(
            "DollsRemote::delete_doll - Sending DELETE request to URL: {}",
            url
        );

        let resp = with_auth(self.client.delete(url)).await.send().await?;

        resp.error_for_status().map_err(|e| {
            tracing::error!("DollsRemote::delete_doll - HTTP error: {}", e);
            e
        })?;

        Ok(())
    }
}
