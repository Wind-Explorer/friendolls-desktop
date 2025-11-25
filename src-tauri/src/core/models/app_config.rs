use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Default, Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct AppConfig {
    pub api_base_url: Option<String>,
}
