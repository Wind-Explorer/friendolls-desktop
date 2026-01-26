use rust_socketio::Payload;
use tauri::async_runtime;
use tracing::error;

use crate::{
    lock_r,
    services::health_manager::show_health_manager_with_error,
    state::FDOLL,
};

use super::WS_EVENT;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStatusPayload {
    pub active_app: String,
    pub state: String,
}

pub async fn report_user_status(status: UserStatusPayload) {
    let payload_value = match serde_json::to_value(&status) {
        Ok(val) => val,
        Err(e) => {
            error!("Failed to serialize user status payload: {}", e);
            return;
        }
    };

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
                WS_EVENT::CLIENT_REPORT_USER_STATUS,
                Payload::Text(vec![payload_value]),
            )
        })
        .await
        {
            Ok(Ok(_)) => (),
            Ok(Err(e)) => {
                error!("Failed to emit user status report: {}", e);
                show_health_manager_with_error(Some(format!(
                    "WebSocket emit failed: {}",
                    e
                )));
            }
            Err(e) => {
                error!("Failed to execute blocking task for user status report: {}", e);
                show_health_manager_with_error(Some(format!(
                    "WebSocket task failed: {}",
                    e
                )));
            }
        }
    }
}
