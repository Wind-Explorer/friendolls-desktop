use once_cell::sync::Lazy;
use tauri::async_runtime::{self, JoinHandle};
use tauri_specta::Event as _;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tracing::warn;

use crate::models::event_payloads::UserStatusPayload;

use crate::services::app_events::UserStatusChanged;

use super::{emitter, types::WS_EVENT};

/// Debouncer for user status reports
static USER_STATUS_REPORT_DEBOUNCE: Lazy<Mutex<Option<JoinHandle<()>>>> =
    Lazy::new(|| Mutex::new(None));

/// Report user status to WebSocket server with debouncing
pub async fn report_user_status(status: UserStatusPayload) -> Result<(), String> {
    let mut debouncer = USER_STATUS_REPORT_DEBOUNCE.lock().await;

    // Cancel previous pending report
    if let Some(handle) = debouncer.take() {
        handle.abort();
    }

    if !status.has_presence_content() {
        return Ok(());
    }

    if let Err(e) = UserStatusChanged(status.clone()).emit(crate::get_app_handle()) {
        warn!("Failed to emit user-status-changed event: {e}");
    }

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
    Ok(())
}
