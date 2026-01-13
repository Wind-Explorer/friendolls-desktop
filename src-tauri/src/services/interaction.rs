use serde_json::json;
use tracing::{error, info};

use crate::{
    lock_r,
    models::interaction::SendInteractionDto,
    services::ws::WS_EVENT,
    state::FDOLL,
};

pub async fn send_interaction(dto: SendInteractionDto) -> Result<(), String> {
    // Check if WS is initialized
    let client = {
        let guard = lock_r!(FDOLL);
        if let Some(clients) = &guard.clients {
            if clients.is_ws_initialized {
                clients.ws_client.clone()
            } else {
                return Err("WebSocket not initialized".to_string());
            }
        } else {
            return Err("App not fully initialized".to_string());
        }
    };

    if let Some(socket) = client {
        // Prepare payload for client-send-interaction event
        // The DTO structure matches what the server expects:
        // { recipientUserId, content, type } (handled by serde rename_all="camelCase")
        
        // Note: The `type` field in DTO is mapped to `type_` in Rust struct but serialized as `type`
        // due to camelCase renaming (if we rely on TS-RS output) or manual renaming.
        // Wait, `type` is a reserved keyword in Rust so we used `type_`.
        // We need to ensure serialization maps `type_` to `type`.
        // However, serde's `rename_all = "camelCase"` handles `recipient_user_id` -> `recipientUserId`.
        // It does NOT automatically handle `type_` -> `type`.
        // We should add `#[serde(rename = "type")]` to the `type_` field in the model.
        // I will fix the model first to ensure correct serialization.
        
        let payload = json!({
            "recipientUserId": dto.recipient_user_id,
            "content": dto.content,
            "type": dto.type_
        });

        info!("Sending interaction via WS: {:?}", payload);

        // Blocking emission because rust_socketio::Client::emit is synchronous/blocking
        // but we are in an async context. Ideally we spawn_blocking.
        let spawn_result = tauri::async_runtime::spawn_blocking(move || {
            socket.emit(WS_EVENT::CLIENT_SEND_INTERACTION, payload)
        }).await;

        match spawn_result {
            Ok(emit_result) => match emit_result {
                Ok(_) => Ok(()),
                Err(e) => {
                    let err_msg = format!("Failed to emit interaction event: {}", e);
                    error!("{}", err_msg);
                    Err(err_msg)
                }
            },
            Err(e) => {
                let err_msg = format!("Failed to spawn blocking task for interaction emit: {}", e);
                error!("{}", err_msg);
                Err(err_msg)
            }
        }
    } else {
        Err("WebSocket client not available".to_string())
    }
}
