use rust_socketio::Payload;
use serde::Serialize;
use tauri::{async_runtime, Emitter};
use tracing::{error, warn};

use crate::{get_app_handle, init::lifecycle::handle_disasterous_failure, lock_r, state::FDOLL};

/// Acquire WebSocket client and initialization state from app state
fn get_ws_state() -> (Option<rust_socketio::client::Client>, bool) {
    let guard = lock_r!(FDOLL);
    if let Some(clients) = &guard.network.clients {
        (
            clients.ws_client.as_ref().cloned(),
            clients.is_ws_initialized,
        )
    } else {
        (None, false)
    }
}

/// Serialize and emit a payload via the WebSocket client (blocking)
async fn do_emit<T: Serialize + Send + 'static>(
    event: &'static str,
    payload: T,
) -> Result<(), String> {
    let (client_opt, is_initialized) = get_ws_state();

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
        Ok(Err(e)) => Err(format!("WebSocket emit failed: {}", e)),
        Err(e) => Err(format!("WebSocket task failed: {}", e)),
    }
}

/// Emit critical data to WebSocket server
///
/// On failure, triggers disaster recovery (session teardown + health manager).
/// Use for essential operations where connection loss is unrecoverable.
#[allow(dead_code)]
pub async fn ws_emit<T: Serialize + Send + 'static>(
    event: &'static str,
    payload: T,
) -> Result<(), String> {
    match do_emit(event, payload).await {
        Ok(_) => Ok(()),
        Err(err_msg) => {
            error!("[critical] {}", err_msg);
            handle_disasterous_failure(Some(err_msg.clone())).await;
            Err(err_msg)
        }
    }
}

/// Emit non-critical data to WebSocket server
///
/// On failure, logs a warning but does NOT trigger disaster recovery.
/// Use for telemetry, status updates, and other non-essential operations
/// where a failure should not tear down the user session.
pub async fn ws_emit_soft<T: Serialize + Send + 'static>(
    event: &'static str,
    payload: T,
) -> Result<(), String> {
    match do_emit(event, payload).await {
        Ok(_) => Ok(()),
        Err(err_msg) => {
            warn!("[non-critical] {}", err_msg);
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
