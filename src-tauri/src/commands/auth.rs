use tauri;
use tracing;

use crate::init::lifecycle;

#[tauri::command]
pub async fn logout_and_restart() -> Result<(), String> {
    crate::services::auth::logout_and_restart()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_auth_flow() -> Result<(), String> {
    // Cancel any in-flight auth listener/state before starting a new one
    crate::services::auth::cancel_auth_flow();

    crate::services::auth::init_auth_code_retrieval(|| {
        tracing::info!("Authentication successful, creating scene...");
        // Close welcome window if it's still open
        crate::services::welcome::close_welcome_window();
        tauri::async_runtime::spawn(async {
            lifecycle::handle_authentication_flow().await;
        });
    })
    .map_err(|e| e.to_string())
}
