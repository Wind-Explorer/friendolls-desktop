use rust_socketio::{Payload, RawClient};
use tracing::info;

use crate::{
    lock_w, services::health_manager::close_health_manager_window,
    services::scene::open_scene_window, state::FDOLL,
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
        mark_ws_initialized();
        restore_connection_ui();
    }
}

/// Mark WebSocket as initialized in app state
fn mark_ws_initialized() {
    let mut guard = lock_w!(FDOLL);
    if let Some(clients) = guard.network.clients.as_mut() {
        clients.is_ws_initialized = true;
    }
}

/// Restore UI after successful connection
fn restore_connection_ui() {
    close_health_manager_window();
    open_scene_window();
}
