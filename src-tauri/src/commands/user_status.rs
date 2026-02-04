use crate::services::active_app::AppMetadata;
use crate::services::ws::UserStatusPayload;
use crate::services::ws::report_user_status;

#[tauri::command]
pub async fn send_user_status_cmd(app_metadata: AppMetadata, state: String) -> Result<(), String> {
    let payload = UserStatusPayload { app_metadata, state };
    report_user_status(payload).await;
    Ok(())
}
