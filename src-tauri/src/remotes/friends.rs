use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{lock_r, services::auth::with_auth, state::FDOLL};

#[derive(Error, Debug)]
pub enum RemoteError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Api(String),
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserBasicDto {
    pub id: String,
    pub name: String,
    pub username: Option<String>,
    pub active_doll_id: Option<String>,
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

    pub async fn get_friends(&self) -> Result<Vec<FriendshipResponseDto>, RemoteError> {
        let url = format!("{}/friends", self.base_url);
        tracing::info!(
            "FriendRemote::get_friends - Sending GET request to URL: {}",
            url
        );
        let resp = with_auth(self.client.get(url)).await.send().await?;
        tracing::info!(
            "FriendRemote::get_friends - Received response with status: {}",
            resp.status()
        );
        let resp = resp.error_for_status().map_err(|e| {
            tracing::error!("FriendRemote::get_friends - HTTP error: {}", e);
            e
        })?;
        tracing::info!(
            "FriendRemote::get_friends - Response status after error_for_status: {}",
            resp.status()
        );
        let text = resp.text().await.map_err(|e| {
            tracing::error!(
                "FriendRemote::get_friends - Failed to read response text: {}",
                e
            );
            e
        })?;
        tracing::info!("FriendRemote::get_friends - Response body: {}", text);
        let friends: Vec<FriendshipResponseDto> = serde_json::from_str(&text).map_err(|e| {
            tracing::error!("FriendRemote::get_friends - Failed to parse JSON: {}", e);
            e
        })?;
        tracing::info!(
            "FriendRemote::get_friends - Successfully parsed {} friends",
            friends.len()
        );
        Ok(friends)
    }

    pub async fn search_users(
        &self,
        username: Option<&str>,
    ) -> Result<Vec<UserBasicDto>, RemoteError> {
        let mut url = format!("{}/friends/search", self.base_url);
        if let Some(u) = username {
            url.push_str(&format!("?username={}", u));
        }
        tracing::info!(
            "FriendRemote::search_users - Sending GET request to URL: {}",
            url
        );
        let resp = with_auth(self.client.get(&url)).await.send().await?;
        tracing::info!(
            "FriendRemote::search_users - Received response with status: {}",
            resp.status()
        );
        let resp = resp.error_for_status().map_err(|e| {
            tracing::error!("FriendRemote::search_users - HTTP error: {}", e);
            e
        })?;
        tracing::info!(
            "FriendRemote::search_users - Response status after error_for_status: {}",
            resp.status()
        );
        let text = resp.text().await.map_err(|e| {
            tracing::error!(
                "FriendRemote::search_users - Failed to read response text: {}",
                e
            );
            e
        })?;
        tracing::info!("FriendRemote::search_users - Response body: {}", text);
        let users: Vec<UserBasicDto> = serde_json::from_str(&text).map_err(|e| {
            tracing::error!("FriendRemote::search_users - Failed to parse JSON: {}", e);
            e
        })?;
        tracing::info!(
            "FriendRemote::search_users - Successfully parsed {} users",
            users.len()
        );
        Ok(users)
    }

    pub async fn send_friend_request(
        &self,
        request: SendFriendRequestDto,
    ) -> Result<FriendRequestResponseDto, RemoteError> {
        let url = format!("{}/friends/requests", self.base_url);
        tracing::info!(
            "FriendRemote::send_friend_request - Sending POST request to URL: {} with body: {:?}",
            url,
            request
        );
        let resp = with_auth(self.client.post(url))
            .await
            .json(&request)
            .send()
            .await?;
        tracing::info!(
            "FriendRemote::send_friend_request - Received response with status: {}",
            resp.status()
        );

        if resp.status() == reqwest::StatusCode::CONFLICT {
            let text = resp.text().await.unwrap_or_default();
            // try to parse the error message
            let error_msg = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| v.get("message").and_then(|m| m.as_str().map(String::from)))
                .unwrap_or_else(|| "Conflict error".to_string());

            return Err(RemoteError::Api(error_msg));
        }

        let resp = resp.error_for_status().map_err(|e| {
            tracing::error!("FriendRemote::send_friend_request - HTTP error: {}", e);
            e
        })?;
        tracing::info!(
            "FriendRemote::send_friend_request - Response status after error_for_status: {}",
            resp.status()
        );
        let text = resp.text().await.map_err(|e| {
            tracing::error!(
                "FriendRemote::send_friend_request - Failed to read response text: {}",
                e
            );
            e
        })?;
        tracing::info!(
            "FriendRemote::send_friend_request - Response body: {}",
            text
        );
        let req_resp: FriendRequestResponseDto = serde_json::from_str(&text).map_err(|e| {
            tracing::error!(
                "FriendRemote::send_friend_request - Failed to parse JSON: {}",
                e
            );
            e
        })?;
        tracing::info!(
            "FriendRemote::send_friend_request - Successfully parsed friend request response"
        );
        Ok(req_resp)
    }

    pub async fn get_received_requests(
        &self,
    ) -> Result<Vec<FriendRequestResponseDto>, RemoteError> {
        let url = format!("{}/friends/requests/received", self.base_url);
        tracing::info!(
            "FriendRemote::get_received_requests - Sending GET request to URL: {}",
            url
        );
        let resp = with_auth(self.client.get(url)).await.send().await?;
        tracing::info!(
            "FriendRemote::get_received_requests - Received response with status: {}",
            resp.status()
        );
        let resp = resp.error_for_status().map_err(|e| {
            tracing::error!("FriendRemote::get_received_requests - HTTP error: {}", e);
            e
        })?;
        tracing::info!(
            "FriendRemote::get_received_requests - Response status after error_for_status: {}",
            resp.status()
        );
        let text = resp.text().await.map_err(|e| {
            tracing::error!(
                "FriendRemote::get_received_requests - Failed to read response text: {}",
                e
            );
            e
        })?;
        tracing::info!(
            "FriendRemote::get_received_requests - Response body: {}",
            text
        );
        let requests: Vec<FriendRequestResponseDto> = serde_json::from_str(&text).map_err(|e| {
            tracing::error!(
                "FriendRemote::get_received_requests - Failed to parse JSON: {}",
                e
            );
            e
        })?;
        tracing::info!(
            "FriendRemote::get_received_requests - Successfully parsed {} received requests",
            requests.len()
        );
        Ok(requests)
    }

    pub async fn get_sent_requests(&self) -> Result<Vec<FriendRequestResponseDto>, RemoteError> {
        let url = format!("{}/friends/requests/sent", self.base_url);
        tracing::info!(
            "FriendRemote::get_sent_requests - Sending GET request to URL: {}",
            url
        );
        let resp = with_auth(self.client.get(url)).await.send().await?;
        tracing::info!(
            "FriendRemote::get_sent_requests - Received response with status: {}",
            resp.status()
        );
        let resp = resp.error_for_status().map_err(|e| {
            tracing::error!("FriendRemote::get_sent_requests - HTTP error: {}", e);
            e
        })?;
        tracing::info!(
            "FriendRemote::get_sent_requests - Response status after error_for_status: {}",
            resp.status()
        );
        let text = resp.text().await.map_err(|e| {
            tracing::error!(
                "FriendRemote::get_sent_requests - Failed to read response text: {}",
                e
            );
            e
        })?;
        tracing::info!("FriendRemote::get_sent_requests - Response body: {}", text);
        let requests: Vec<FriendRequestResponseDto> = serde_json::from_str(&text).map_err(|e| {
            tracing::error!(
                "FriendRemote::get_sent_requests - Failed to parse JSON: {}",
                e
            );
            e
        })?;
        tracing::info!(
            "FriendRemote::get_sent_requests - Successfully parsed {} sent requests",
            requests.len()
        );
        Ok(requests)
    }

    pub async fn accept_friend_request(
        &self,
        request_id: &str,
    ) -> Result<FriendRequestResponseDto, RemoteError> {
        let url = format!("{}/friends/requests/{}/accept", self.base_url, request_id);
        tracing::info!(
            "FriendRemote::accept_friend_request - Sending POST request to URL: {}",
            url
        );
        let resp = with_auth(self.client.post(url)).await.send().await?;
        tracing::info!(
            "FriendRemote::accept_friend_request - Received response with status: {}",
            resp.status()
        );
        let resp = resp.error_for_status().map_err(|e| {
            tracing::error!("FriendRemote::accept_friend_request - HTTP error: {}", e);
            e
        })?;
        tracing::info!(
            "FriendRemote::accept_friend_request - Response status after error_for_status: {}",
            resp.status()
        );
        let text = resp.text().await.map_err(|e| {
            tracing::error!(
                "FriendRemote::accept_friend_request - Failed to read response text: {}",
                e
            );
            e
        })?;
        tracing::info!(
            "FriendRemote::accept_friend_request - Response body: {}",
            text
        );
        let req_resp: FriendRequestResponseDto = serde_json::from_str(&text).map_err(|e| {
            tracing::error!(
                "FriendRemote::accept_friend_request - Failed to parse JSON: {}",
                e
            );
            e
        })?;
        tracing::info!(
            "FriendRemote::accept_friend_request - Successfully parsed friend request response"
        );
        Ok(req_resp)
    }

    pub async fn deny_friend_request(
        &self,
        request_id: &str,
    ) -> Result<FriendRequestResponseDto, RemoteError> {
        let url = format!("{}/friends/requests/{}/deny", self.base_url, request_id);
        tracing::info!(
            "FriendRemote::deny_friend_request - Sending POST request to URL: {}",
            url
        );
        let resp = with_auth(self.client.post(url)).await.send().await?;
        tracing::info!(
            "FriendRemote::deny_friend_request - Received response with status: {}",
            resp.status()
        );
        let resp = resp.error_for_status().map_err(|e| {
            tracing::error!("FriendRemote::deny_friend_request - HTTP error: {}", e);
            e
        })?;
        tracing::info!(
            "FriendRemote::deny_friend_request - Response status after error_for_status: {}",
            resp.status()
        );
        let text = resp.text().await.map_err(|e| {
            tracing::error!(
                "FriendRemote::deny_friend_request - Failed to read response text: {}",
                e
            );
            e
        })?;
        tracing::info!(
            "FriendRemote::deny_friend_request - Response body: {}",
            text
        );
        let req_resp: FriendRequestResponseDto = serde_json::from_str(&text).map_err(|e| {
            tracing::error!(
                "FriendRemote::deny_friend_request - Failed to parse JSON: {}",
                e
            );
            e
        })?;
        tracing::info!(
            "FriendRemote::deny_friend_request - Successfully parsed friend request response"
        );
        Ok(req_resp)
    }

    pub async fn unfriend(&self, friend_id: &str) -> Result<(), RemoteError> {
        let url = format!("{}/friends/{}", self.base_url, friend_id);
        tracing::info!(
            "FriendRemote::unfriend - Sending DELETE request to URL: {}",
            url
        );
        let resp = with_auth(self.client.delete(url)).await.send().await?;
        tracing::info!(
            "FriendRemote::unfriend - Received response with status: {}",
            resp.status()
        );
        resp.error_for_status().map_err(|e| {
            tracing::error!("FriendRemote::unfriend - HTTP error: {}", e);
            e
        })?;
        tracing::info!("FriendRemote::unfriend - Successfully unfriended");
        Ok(())
    }
}
