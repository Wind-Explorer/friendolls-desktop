use once_cell::sync::Lazy;
use serde::Serialize;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::Duration;
use tracing::warn;

use crate::services::active_app::AppMetadata;

use super::{emitter, types::WS_EVENT};

/// User status payload sent to WebSocket server
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStatusPayload {
    pub app_metadata: AppMetadata,
    pub state: String,
}

/// Debouncer for user status reports
static USER_STATUS_REPORT_DEBOUNCE: Lazy<Mutex<Option<JoinHandle<()>>>> =
    Lazy::new(|| Mutex::new(None));

/// Report user status to WebSocket server with debouncing
///
/// Uses soft emit to avoid triggering disaster recovery on failure,
/// since user status is non-critical telemetry.
pub async fn report_user_status(status: UserStatusPayload) {
    let mut debouncer = USER_STATUS_REPORT_DEBOUNCE.lock().await;

    // Cancel previous pending report
    if let Some(handle) = debouncer.take() {
        handle.abort();
    }

    // Schedule new report after 500ms
    let handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        if let Err(e) = emitter::ws_emit_soft(WS_EVENT::CLIENT_REPORT_USER_STATUS, status).await {
            warn!("User status report failed: {}", e);
        }
    });

    *debouncer = Some(handle);
}
