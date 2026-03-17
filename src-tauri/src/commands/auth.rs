use crate::services::auth;

#[tauri::command]
#[specta::specta]
pub async fn logout_and_restart() -> Result<(), String> {
    auth::logout_and_restart().await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn start_google_auth() -> Result<(), String> {
    auth::start_browser_login("google")
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn start_discord_auth() -> Result<(), String> {
    auth::start_browser_login("discord")
        .await
        .map_err(|e| e.to_string())
}
