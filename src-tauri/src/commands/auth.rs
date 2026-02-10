use crate::services::auth;

#[tauri::command]
pub async fn logout_and_restart() -> Result<(), String> {
    auth::logout_and_restart().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn login(email: String, password: String) -> Result<(), String> {
    auth::login_and_init_session(&email, &password)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn register(
    email: String,
    password: String,
    name: Option<String>,
    username: Option<String>,
) -> Result<String, String> {
    auth::register(
        &email,
        &password,
        name.as_deref(),
        username.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn change_password(
    current_password: String,
    new_password: String,
) -> Result<(), String> {
    auth::change_password(&current_password, &new_password)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_password(old_password: String, new_password: String) -> Result<(), String> {
    auth::reset_password(&old_password, &new_password)
        .await
        .map_err(|e| e.to_string())
}
