use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
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
