use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::dolls::DollDto;
use super::friends::UserBasicDto;
use crate::services::presence_modules::models::PresenceStatus;

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum UserStatusState {
    Idle,
    Resting,
}

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserStatusPayload {
    pub presence_status: PresenceStatus,
    pub state: UserStatusState,
}

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FriendUserStatusPayload {
    pub user_id: String,
    pub status: UserStatusPayload,
}

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FriendDisconnectedPayload {
    pub user_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FriendActiveDollChangedPayload {
    pub friend_id: String,
    pub doll: Option<DollDto>,
}

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FriendRequestReceivedPayload {
    pub id: String,
    pub sender: UserBasicDto,
    pub created_at: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FriendRequestAcceptedPayload {
    pub id: String,
    pub friend: UserBasicDto,
    pub accepted_at: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FriendRequestDeniedPayload {
    pub id: String,
    pub denier: UserBasicDto,
    pub denied_at: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UnfriendedPayload {
    pub friend_id: String,
}
