use reqwest::Error;
use serde_json::json;

use crate::services::auth::with_auth;
use crate::{lock_r, state::FDOLL};

pub struct SessionRemote {
    pub base_url: String,
    pub client: reqwest::Client,
}

impl SessionRemote {
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

    pub async fn logout(
        &self,
        refresh_token: &str,
        session_state: Option<&str>,
    ) -> Result<(), Error> {
        let url = format!("{}/users/logout", self.base_url);
        let body = json!({
            "refreshToken": refresh_token,
            "sessionState": session_state,
        });
        let resp = with_auth(self.client.post(url))
            .await
            .json(&body)
            .send()
            .await?;
        resp.error_for_status()?;
        Ok(())
    }
}
