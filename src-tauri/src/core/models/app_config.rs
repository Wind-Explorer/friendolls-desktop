use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct AuthConfig {
    pub audience: String,
    pub auth_url: String,
    pub redirect_uri: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct AppConfig {
    pub api_base_url: Option<String>,
    pub auth: AuthConfig,
}
