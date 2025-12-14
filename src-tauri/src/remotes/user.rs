use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{lock_r, services::auth::with_auth, state::FDOLL};

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserProfile {
    pub id: String,
    pub keycloak_sub: String,
    pub name: String,
    pub email: String,
    pub username: Option<String>,

    pub roles: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_login_at: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateUserDto {
    // Empty as per API schema
}

pub struct UserRemote {
    pub base_url: String,
    pub client: Client,
}

impl UserRemote {
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

    pub async fn get_user(&self, user_id: Option<&str>) -> Result<UserProfile, Error> {
        let url = format!("{}/users/{}", self.base_url, user_id.unwrap_or("me"));
        let resp = with_auth(self.client.get(url)).await.send().await?;
        let user = resp.json().await?;
        Ok(user)
    }

    pub async fn update_user(
        &self,
        user_id: Option<&str>,
        update: UpdateUserDto,
    ) -> Result<UserProfile, Error> {
        let url = format!("{}/users/{}", self.base_url, user_id.unwrap_or("me"));
        let resp = with_auth(self.client.put(url))
            .await
            .json(&update)
            .send()
            .await?;
        let user = resp.json().await?;
        Ok(user)
    }

    pub async fn delete_user(&self, user_id: Option<&str>) -> Result<(), Error> {
        let url = format!("{}/users/{}", self.base_url, user_id.unwrap_or("me"));
        let resp = with_auth(self.client.delete(url)).await.send().await?;
        resp.error_for_status()?;
        Ok(())
    }
}
