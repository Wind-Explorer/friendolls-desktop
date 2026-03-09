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

impl UserStatusPayload {
    pub fn has_presence_content(&self) -> bool {
        self.presence_status
            .title
            .as_ref()
            .is_some_and(|title| !title.trim().is_empty())
            || self
                .presence_status
                .subtitle
                .as_ref()
                .is_some_and(|subtitle| !subtitle.trim().is_empty())
    }
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
