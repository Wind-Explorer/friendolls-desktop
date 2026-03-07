use serde::{Deserialize, Serialize};
use specta::Type;

use super::dolls::DollDto;

#[derive(Default, Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct UserBasicDto {
    pub id: String,
    pub name: String,
    pub username: Option<String>,
    pub active_doll: Option<DollDto>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct FriendshipResponseDto {
    pub id: String,
    pub friend: Option<UserBasicDto>,
    pub created_at: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct SendFriendRequestDto {
    pub receiver_id: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct FriendRequestResponseDto {
    pub id: String,
    pub sender: UserBasicDto,
    pub receiver: UserBasicDto,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
