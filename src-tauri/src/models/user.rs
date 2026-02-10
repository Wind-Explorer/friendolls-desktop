use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub email: String,
    pub username: Option<String>,

    pub roles: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_login_at: Option<String>,
    pub active_doll_id: Option<String>,
}
