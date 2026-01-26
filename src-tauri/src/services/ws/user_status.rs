use once_cell::sync::Lazy;
use rust_socketio::Payload;
use tauri::async_runtime;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::Duration;
use tracing::error;

use crate::{lock_r, services::health_manager::show_health_manager_with_error, state::FDOLL};

use super::WS_EVENT;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStatusPayload {
    pub active_app: String,
    pub state: String,
}

static USER_STATUS_REPORT_DEBOUNCE: Lazy<Mutex<Option<JoinHandle<()>>>> =
    Lazy::new(|| Mutex::new(None));

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

    {
        let mut debouncer = USER_STATUS_REPORT_DEBOUNCE.lock().await;
        if let Some(handle) = debouncer.take() {
            handle.abort();
        }
        let payload_value_clone = payload_value.clone();
        let client_opt_clone = client_opt.clone();
        let is_initialized_clone = is_initialized;
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if let Some(client) = client_opt_clone {
                if !is_initialized_clone {
                    return;
                }
                match async_runtime::spawn_blocking(move || {
                    client.emit(
                        WS_EVENT::CLIENT_REPORT_USER_STATUS,
                        Payload::Text(vec![payload_value_clone]),
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
                        error!(
                            "Failed to execute blocking task for user status report: {}",
                            e
                        );
                        show_health_manager_with_error(Some(format!(
                            "WebSocket task failed: {}",
                            e
                        )));
                    }
                }
            }
        });
        *debouncer = Some(handle);
    }
}
