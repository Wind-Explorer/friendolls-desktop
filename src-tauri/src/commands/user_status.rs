use crate::services::ws::UserStatusPayload;
use crate::services::ws::report_user_status;

#[tauri::command]
pub async fn send_user_status_cmd(active_app: String, state: String) -> Result<(), String> {
    let payload = UserStatusPayload { active_app, state };
    report_user_status(payload).await;
    Ok(())
}
