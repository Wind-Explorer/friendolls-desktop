use rust_socketio::Payload;
use serde::Serialize;
use tauri::{async_runtime, Emitter};
use tracing::error;

use crate::{get_app_handle, init::lifecycle::handle_disasterous_failure, lock_r, state::FDOLL};

/// Emit data to WebSocket server
///
/// Handles client acquisition, initialization checks, blocking emit, and error handling.
/// Returns Ok(()) on success, Err with message on failure.
pub async fn ws_emit<T: Serialize + Send + 'static>(
    event: &'static str,
    payload: T,
) -> Result<(), String> {
    let (client_opt, is_initialized) = {
        let guard = lock_r!(FDOLL);
        if let Some(clients) = &guard.network.clients {
            (
                clients.ws_client.as_ref().cloned(),
                clients.is_ws_initialized,
            )
        } else {
            (None, false)
        }
    };

    let Some(client) = client_opt else {
        return Ok(()); // Client not available, silent skip
    };

    if !is_initialized {
        return Ok(()); // Not initialized yet, silent skip
    }

    let payload_value = serde_json::to_value(&payload)
        .map_err(|e| format!("Failed to serialize payload: {}", e))?;

    match async_runtime::spawn_blocking(move || {
        client.emit(event, Payload::Text(vec![payload_value]))
    })
    .await
    {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => {
            let err_msg = format!("WebSocket emit failed: {}", e);
            error!("{}", err_msg);
            handle_disasterous_failure(Some(err_msg.clone())).await;
            Err(err_msg)
        }
        Err(e) => {
            let err_msg = format!("WebSocket task failed: {}", e);
            error!("Failed to execute blocking task for {}: {}", event, e);
            handle_disasterous_failure(Some(err_msg.clone())).await;
            Err(err_msg)
        }
    }
}

/// Emit event to frontend (Tauri window)
///
/// Handles error logging consistently.
pub fn emit_to_frontend<T: Serialize + Clone>(event: &str, payload: T) {
    if let Err(e) = get_app_handle().emit(event, payload) {
        error!("Failed to emit {} event to frontend: {:?}", event, e);
    }
}
