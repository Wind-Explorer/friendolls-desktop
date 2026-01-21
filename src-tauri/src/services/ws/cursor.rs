use rust_socketio::Payload;
use tauri::async_runtime;
use tracing::error;

use crate::{
    lock_r,
    services::{cursor::CursorPosition, health_manager::show_health_manager_with_error},
    state::FDOLL,
};

use super::WS_EVENT;

pub async fn report_cursor_data(cursor_position: CursorPosition) {
    // Only attempt to get clients if lock_r succeeds (it should, but safety first)
    // and if clients are actually initialized.
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

    if let Some(client) = client_opt {
        if !is_initialized {
            return;
        }

        match async_runtime::spawn_blocking(move || {
            client.emit(
                WS_EVENT::CURSOR_REPORT_POSITION,
                Payload::Text(vec![serde_json::json!(cursor_position)]),
            )
        })
        .await
        {
            Ok(Ok(_)) => (),
            Ok(Err(e)) => {
                error!("Failed to emit cursor report: {}", e);
                show_health_manager_with_error(Some(format!("WebSocket emit failed: {}", e)));
            }
            Err(e) => {
                error!("Failed to execute blocking task for cursor report: {}", e);
                show_health_manager_with_error(Some(format!("WebSocket task failed: {}", e)));
            }
        }
    }
}
