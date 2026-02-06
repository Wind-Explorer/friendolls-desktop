use rust_socketio::{Payload, RawClient};
use tracing::info;

use crate::{
    lock_w,
    services::health_manager::close_health_manager_window,
    services::scene::open_scene_window,
    state::{init_app_data_scoped, AppDataRefreshScope, FDOLL},
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
        let needs_data_refresh = check_and_mark_initialized();
        restore_connection_ui();

        if needs_data_refresh {
            info!("Reconnection detected: refreshing app data");
            tauri::async_runtime::spawn(async {
                init_app_data_scoped(AppDataRefreshScope::All).await;
            });
        }
    }
}

/// Mark WebSocket as initialized and check if app data needs refreshing
///
/// Returns true if user data is missing (indicating a reconnection
/// after session teardown where app data was cleared).
fn check_and_mark_initialized() -> bool {
    let mut guard = lock_w!(FDOLL);
    if let Some(clients) = guard.network.clients.as_mut() {
        clients.is_ws_initialized = true;
    }
    // If user data is gone, we need to re-fetch everything
    guard.user_data.user.is_none()
}

/// Restore UI after successful connection
fn restore_connection_ui() {
    close_health_manager_window();
    open_scene_window();
}
