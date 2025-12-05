use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct AuthConfig {
    pub audience: String,
    pub auth_url: String,
    pub redirect_uri: String,
    pub redirect_host: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub api_base_url: Option<String>,
    pub auth: AuthConfig,
}
