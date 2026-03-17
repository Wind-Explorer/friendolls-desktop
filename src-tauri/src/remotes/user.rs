use reqwest::{Client, Error};

use crate::{lock_r, models::user::*, services::auth::with_auth, state::FDOLL};

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
                .network
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

    pub async fn set_active_doll(&self, doll_id: &str) -> Result<(), Error> {
        let url = format!("{}/users/me/active-doll/{}", self.base_url, doll_id);
        let resp = with_auth(self.client.put(url)).await.send().await?;
        resp.error_for_status()?;
        Ok(())
    }

    pub async fn remove_active_doll(&self) -> Result<(), Error> {
        let url = format!("{}/users/me/active-doll", self.base_url);
        let resp = with_auth(self.client.delete(url)).await.send().await?;
        resp.error_for_status()?;
        Ok(())
    }
}
