use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::remotes::{friends::FriendshipResponseDto, user::UserProfile};

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct AppData {
    pub user: Option<UserProfile>,
    pub friends: Option<Vec<FriendshipResponseDto>>,
}
