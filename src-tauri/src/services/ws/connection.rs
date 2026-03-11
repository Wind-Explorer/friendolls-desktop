use rust_socketio::{Payload, RawClient};
use tracing::info;

use crate::{
    lock_w,
    services::{health_manager::close_health_manager_window, session::construct_user_session},
    state::FDOLL,
};

use super::{types::WS_EVENT, utils};

/// Emit initialization request to WebSocket server
fn emit_initialize(socket: &RawClient) {
    if let Err(e) = socket.emit(WS_EVENT::CLIENT_INITIALIZE, serde_json::json!({})) {
        tracing::error!("Failed to emit client-initialize: {:?}", e);
    }
}

/// Handler for WebSocket connection event
pub fn on_connected(_payload: Payload, socket: RawClient) {
    info!("WebSocket connected. Sending initialization request.");
    emit_initialize(&socket);
}

/// Handler for initialized event
pub fn on_initialized(payload: Payload, _socket: RawClient) {
    if utils::extract_text_value(payload, "initialized").is_ok() {
        let is_reconnection = mark_ws_initialized();

        if is_reconnection {
            info!("Reconnection detected: reconstructing user session");
            tauri::async_runtime::spawn(async {
                construct_user_session().await;
            });
        } else {
            // First-time initialization, just close health manager if open
            close_health_manager_window();
        }
    }
}

/// Mark WebSocket as initialized and check if this is a reconnection.
///
/// Returns true if user data is missing (indicating a reconnection
/// after session teardown where app data was cleared).
fn mark_ws_initialized() -> bool {
    let mut guard = lock_w!(FDOLL);
    if let Some(clients) = guard.network.clients.as_mut() {
        clients.is_ws_initialized = true;
        clients.ws_emit_failures = 0;
    }
    // If user data is gone, we need full session reconstruction
    guard.user_data.user.is_none()
}
