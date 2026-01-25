use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::dolls::DollDto;

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserBasicDto {
    pub id: String,
    pub name: String,
    pub username: Option<String>,
    pub active_doll: Option<DollDto>,
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