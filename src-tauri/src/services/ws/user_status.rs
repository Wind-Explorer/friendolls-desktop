use once_cell::sync::Lazy;
use serde::Serialize;
use tauri::async_runtime::{self, JoinHandle};
use tokio::sync::Mutex;
use tokio::time::Duration;
use tracing::warn;

use crate::services::presence_modules::models::PresenceStatus;

use super::{emitter, types::WS_EVENT};

/// User status payload sent to WebSocket server
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStatusPayload {
    pub presence_status: PresenceStatus,
    pub state: String,
}

pub static USER_STATUS_CHANGED: &str = "user-status-changed";

/// Debouncer for user status reports
static USER_STATUS_REPORT_DEBOUNCE: Lazy<Mutex<Option<JoinHandle<()>>>> =
    Lazy::new(|| Mutex::new(None));

/// Report user status to WebSocket server with debouncing
pub async fn report_user_status(status: UserStatusPayload) {
    let mut debouncer = USER_STATUS_REPORT_DEBOUNCE.lock().await;

    // Cancel previous pending report
    if let Some(handle) = debouncer.take() {
        handle.abort();
    }

    emitter::emit_to_frontend(USER_STATUS_CHANGED, &status);

    // Schedule new report after 500ms
    let handle = async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        if let Err(e) =
            emitter::ws_emit_soft(WS_EVENT::CLIENT_REPORT_USER_STATUS, status.clone()).await
        {
            warn!("User status report failed: {}", e);
        };
    });

    *debouncer = Some(handle);
}
