use rust_socketio::{Payload, RawClient};

use crate::models::interaction::{InteractionDeliveryFailedDto, InteractionPayloadDto};
use crate::services::app_events::AppEvents;

use super::{emitter, utils};

/// Handler for interaction-received event
pub fn on_interaction_received(payload: Payload, _socket: RawClient) {
    if let Ok(data) =
        utils::extract_and_parse::<InteractionPayloadDto>(payload, "interaction-received")
    {
        emitter::emit_to_frontend(AppEvents::InteractionReceived.as_str(), data);
    }
}

/// Handler for interaction-delivery-failed event
pub fn on_interaction_delivery_failed(payload: Payload, _socket: RawClient) {
    if let Ok(data) = utils::extract_and_parse::<InteractionDeliveryFailedDto>(
        payload,
        "interaction-delivery-failed",
    ) {
        emitter::emit_to_frontend(AppEvents::InteractionDeliveryFailed.as_str(), data);
    }
}
