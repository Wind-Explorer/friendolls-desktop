use crate::{models::interaction::SendInteractionDto, services::interaction::send_interaction};

#[tauri::command]
pub async fn send_interaction_cmd(dto: SendInteractionDto) -> Result<(), String> {
    send_interaction(dto).await
}
