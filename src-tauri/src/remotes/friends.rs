use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{lock_r, services::auth::with_auth, state::FDOLL};

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserBasicDto {
    pub id: String,
    pub name: String,
    pub username: String,
    pub picture: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FriendshipResponseDto {
    pub id: String,
    pub friend: UserBasicDto,
    pub created_at: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SendFriendRequestDto {
    pub receiver_id: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FriendRequestResponseDto {
    pub id: String,
    pub sender: UserBasicDto,
    pub receiver: UserBasicDto,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct FriendRemote {
    pub base_url: String,
    pub client: Client,
}

impl FriendRemote {
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

    pub async fn get_friends(&self) -> Result<Vec<FriendshipResponseDto>, Error> {
        let url = format!("{}/friends", self.base_url);
        let resp = with_auth(self.client.get(url)).await.send().await?;
        let friends = resp.json().await?;
        Ok(friends)
    }

    pub async fn search_users(&self, username: Option<&str>) -> Result<Vec<UserBasicDto>, Error> {
        let mut url = format!("{}/friends/search", self.base_url);
        if let Some(u) = username {
            url.push_str(&format!("?username={}", u));
        }
        let resp = with_auth(self.client.get(&url)).await.send().await?;
        let users = resp.json().await?;
        Ok(users)
    }

    pub async fn send_friend_request(
        &self,
        request: SendFriendRequestDto,
    ) -> Result<FriendRequestResponseDto, Error> {
        let url = format!("{}/friends/requests", self.base_url);
        let resp = with_auth(self.client.post(url))
            .await
            .json(&request)
            .send()
            .await?;
        let req_resp = resp.json().await?;
        Ok(req_resp)
    }

    pub async fn get_received_requests(&self) -> Result<Vec<FriendRequestResponseDto>, Error> {
        let url = format!("{}/friends/requests/received", self.base_url);
        let resp = with_auth(self.client.get(url)).await.send().await?;
        let requests = resp.json().await?;
        Ok(requests)
    }

    pub async fn get_sent_requests(&self) -> Result<Vec<FriendRequestResponseDto>, Error> {
        let url = format!("{}/friends/requests/sent", self.base_url);
        let resp = with_auth(self.client.get(url)).await.send().await?;
        let requests = resp.json().await?;
        Ok(requests)
    }

    pub async fn accept_friend_request(
        &self,
        request_id: &str,
    ) -> Result<FriendRequestResponseDto, Error> {
        let url = format!("{}/friends/requests/{}/accept", self.base_url, request_id);
        let resp = with_auth(self.client.post(url)).await.send().await?;
        let req_resp = resp.json().await?;
        Ok(req_resp)
    }

    pub async fn deny_friend_request(
        &self,
        request_id: &str,
    ) -> Result<FriendRequestResponseDto, Error> {
        let url = format!("{}/friends/requests/{}/deny", self.base_url, request_id);
        let resp = with_auth(self.client.post(url)).await.send().await?;
        let req_resp = resp.json().await?;
        Ok(req_resp)
    }

    pub async fn unfriend(&self, friend_id: &str) -> Result<(), Error> {
        let url = format!("{}/friends/{}", self.base_url, friend_id);
        let resp = with_auth(self.client.delete(url)).await.send().await?;
        resp.error_for_status()?;
        Ok(())
    }
}
