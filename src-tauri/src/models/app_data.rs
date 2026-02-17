use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::models::{dolls::DollDto, friends::FriendshipResponseDto, user::UserProfile};

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct DisplayData {
    pub screen_width: i32,
    pub screen_height: i32,
    pub monitor_scale_factor: f64,
}

impl Default for DisplayData {
    fn default() -> Self {
        Self {
            screen_width: 0,
            screen_height: 0,
            monitor_scale_factor: 1.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct SceneData {
    pub display: DisplayData,
    pub grid_size: i32,
}

impl Default for SceneData {
    fn default() -> Self {
        Self {
            display: DisplayData::default(),
            grid_size: 600,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct UserData {
    pub user: Option<UserProfile>,
    pub friends: Option<Vec<FriendshipResponseDto>>,
    pub dolls: Option<Vec<DollDto>>,
    pub scene: SceneData, // TODO: move this out of app data
}
