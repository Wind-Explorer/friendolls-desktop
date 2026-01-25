use rust_socketio::{Payload, RawClient};
use tracing::info;

use crate::{
    lock_w,
    services::health_manager::close_health_manager_window,
    services::scene::open_scene_window,
    state::FDOLL,
};

use super::WS_EVENT;

fn emit_initialize(socket: &RawClient) {
    if let Err(e) = socket.emit(WS_EVENT::CLIENT_INITIALIZE, serde_json::json!({})) {
        tracing::error!("Failed to emit client-initialize: {:?}", e);
    }
}

pub fn on_connected(_payload: Payload, socket: RawClient) {
    info!("WebSocket connected. Sending initialization request.");
    emit_initialize(&socket);
}

pub fn on_initialized(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                info!("Received initialized event: {:?}", first_value);

                // Mark WebSocket as initialized and reset backoff timer
                let mut guard = lock_w!(FDOLL);
                if let Some(clients) = guard.network.clients.as_mut() {
                    clients.is_ws_initialized = true;
                }

                // Connection restored: close health manager and reopen scene
                close_health_manager_window();
                open_scene_window();
            } else {
                info!("Received initialized event with empty payload");
            }
        }
        _ => tracing::error!("Received unexpected payload format for initialized"),
    }
}