use crate::get_app_handle;
use crate::init::lifecycle::{construct_user_session, validate_server_health};
use crate::services::auth::get_session_token;
use tracing::info;

#[tauri::command]
#[specta::specta]
pub fn quit_app() -> Result<(), String> {
    let app_handle = get_app_handle();
    app_handle.exit(0);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn restart_app() {
    let app_handle = get_app_handle();
    app_handle.restart();
}

/// Attempt to re-establish the user session without restarting the app.
///
/// Validates server health, checks for a valid session token,
/// then reconstructs the user session (re-fetches app data + WebSocket).
#[tauri::command]
#[specta::specta]
pub async fn retry_connection() -> Result<(), String> {
    info!("Retrying connection...");

    validate_server_health()
        .await
        .map_err(|e| format!("Server health check failed: {}", e))?;

    if get_session_token().await.is_none() {
        return Err("No valid session token. Please restart and log in again.".to_string());
    }

    construct_user_session().await;
    info!("Connection retry succeeded");
    Ok(())
}
