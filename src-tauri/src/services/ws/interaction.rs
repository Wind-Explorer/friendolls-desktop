use rust_socketio::{Payload, RawClient};
use tauri::Emitter;
use tracing::{error, info};

use crate::{
    get_app_handle,
    models::interaction::{InteractionDeliveryFailedDto, InteractionPayloadDto},
};

use super::WS_EVENT;

pub fn on_interaction_received(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                info!("Received interaction-received event: {:?}", first_value);

                let interaction_data: Result<InteractionPayloadDto, _> =
                    serde_json::from_value(first_value.clone());

                match interaction_data {
                    Ok(data) => {
                        if let Err(e) = get_app_handle().emit(WS_EVENT::INTERACTION_RECEIVED, data)
                        {
                            error!("Failed to emit interaction-received event: {:?}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse interaction payload: {}", e);
                    }
                }
            } else {
                info!("Received interaction-received event with empty payload");
            }
        }
        _ => error!("Received unexpected payload format for interaction-received"),
    }
}

pub fn on_interaction_delivery_failed(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                info!(
                    "Received interaction-delivery-failed event: {:?}",
                    first_value
                );

                let failure_data: Result<InteractionDeliveryFailedDto, _> =
                    serde_json::from_value(first_value.clone());

                match failure_data {
                    Ok(data) => {
                        if let Err(e) =
                            get_app_handle().emit(WS_EVENT::INTERACTION_DELIVERY_FAILED, data)
                        {
                            error!("Failed to emit interaction-delivery-failed event: {:?}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse interaction failure payload: {}", e);
                    }
                }
            } else {
                info!("Received interaction-delivery-failed event with empty payload");
            }
        }
        _ => error!("Received unexpected payload format for interaction-delivery-failed"),
    }
}
