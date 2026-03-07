use serde::{Deserialize, Serialize};
use specta::Type;

use super::dolls::DollDto;
use super::friends::UserBasicDto;
use crate::services::presence_modules::models::PresenceStatus;

#[derive(Clone, Serialize, Deserialize, Debug, Type)]
#[serde(rename_all = "lowercase")]
pub enum UserStatusState {
    Idle,
    Resting,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct UserStatusPayload {
    pub presence_status: PresenceStatus,
    pub state: UserStatusState,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct FriendUserStatusPayload {
    pub user_id: String,
    pub status: UserStatusPayload,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct FriendDisconnectedPayload {
    pub user_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct FriendActiveDollChangedPayload {
    pub friend_id: String,
    pub doll: Option<DollDto>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct FriendRequestReceivedPayload {
    pub id: String,
    pub sender: UserBasicDto,
    pub created_at: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct FriendRequestAcceptedPayload {
    pub id: String,
    pub friend: UserBasicDto,
    pub accepted_at: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct FriendRequestDeniedPayload {
    pub id: String,
    pub denier: UserBasicDto,
    pub denied_at: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct UnfriendedPayload {
    pub friend_id: String,
}
