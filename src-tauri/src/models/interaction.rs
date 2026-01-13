use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SendInteractionDto {
    pub recipient_user_id: String,
    pub content: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct InteractionPayloadDto {
    pub sender_user_id: String,
    pub sender_name: String,
    pub content: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub timestamp: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct InteractionDeliveryFailedDto {
    pub recipient_user_id: String,
    pub reason: String,
}
