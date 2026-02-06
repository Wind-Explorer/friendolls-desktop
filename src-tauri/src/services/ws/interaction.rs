use rust_socketio::{Payload, RawClient};

use crate::models::interaction::{InteractionDeliveryFailedDto, InteractionPayloadDto};

use super::{emitter, types::WS_EVENT, utils};

/// Handler for interaction-received event
pub fn on_interaction_received(payload: Payload, _socket: RawClient) {
    if let Ok(data) =
        utils::extract_and_parse::<InteractionPayloadDto>(payload, "interaction-received")
    {
        emitter::emit_to_frontend(WS_EVENT::INTERACTION_RECEIVED, data);
    }
}

/// Handler for interaction-delivery-failed event
pub fn on_interaction_delivery_failed(payload: Payload, _socket: RawClient) {
    if let Ok(data) = utils::extract_and_parse::<InteractionDeliveryFailedDto>(
        payload,
        "interaction-delivery-failed",
    ) {
        emitter::emit_to_frontend(WS_EVENT::INTERACTION_DELIVERY_FAILED, data);
    }
}
