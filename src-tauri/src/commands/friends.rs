use crate::remotes::friends::FriendRemote;
use crate::models::friends::{
    FriendRequestResponseDto, FriendshipResponseDto, SendFriendRequestDto, UserBasicDto,
};

#[tauri::command]
pub async fn list_friends() -> Result<Vec<FriendshipResponseDto>, String> {
    FriendRemote::new()
        .get_friends()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_users(username: Option<String>) -> Result<Vec<UserBasicDto>, String> {
    tracing::info!(
        "Tauri command search_users called with username: {:?}",
        username
    );
    let remote = FriendRemote::new();
    tracing::info!("FriendRemote instance created for search_users");
    let result = remote.search_users(username.as_deref()).await;
    tracing::info!("FriendRemote::search_users result: {:?}", result);
    result.map_err(|e| {
        tracing::error!("search_users command error: {}", e);
        e.to_string()
    })
}

#[tauri::command]
pub async fn send_friend_request(
    request: SendFriendRequestDto,
) -> Result<FriendRequestResponseDto, String> {
    FriendRemote::new()
        .send_friend_request(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn received_friend_requests() -> Result<Vec<FriendRequestResponseDto>, String> {
    FriendRemote::new()
        .get_received_requests()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sent_friend_requests() -> Result<Vec<FriendRequestResponseDto>, String> {
    FriendRemote::new()
        .get_sent_requests()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn accept_friend_request(request_id: String) -> Result<FriendRequestResponseDto, String> {
    FriendRemote::new()
        .accept_friend_request(&request_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn deny_friend_request(request_id: String) -> Result<FriendRequestResponseDto, String> {
    FriendRemote::new()
        .deny_friend_request(&request_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unfriend(friend_id: String) -> Result<(), String> {
    FriendRemote::new()
        .unfriend(&friend_id)
        .await
        .map_err(|e| e.to_string())
}
