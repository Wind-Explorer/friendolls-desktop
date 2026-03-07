use crate::{
    models::dolls::{CreateDollDto, DollDto, UpdateDollDto},
    remotes::{
        dolls::DollsRemote,
        user::UserRemote,
    },
    state::AppDataRefreshScope,
    commands::{refresh_app_data, refresh_app_data_conditionally, is_active_doll},
};

#[tauri::command]
#[specta::specta]
pub async fn get_dolls() -> Result<Vec<DollDto>, String> {
    DollsRemote::new()
        .get_dolls()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_doll(id: String) -> Result<DollDto, String> {
    DollsRemote::new()
        .get_doll(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn create_doll(dto: CreateDollDto) -> Result<DollDto, String> {
    let result = DollsRemote::new()
        .create_doll(dto)
        .await
        .map_err(|e| e.to_string())?;

    refresh_app_data(&[AppDataRefreshScope::Dolls]).await;

    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn update_doll(id: String, dto: UpdateDollDto) -> Result<DollDto, String> {
    let result = DollsRemote::new()
        .update_doll(&id, dto)
        .await
        .map_err(|e| e.to_string())?;

    // Check if this was the active doll (after update completes to avoid stale reads)
    let is_active = is_active_doll(&id);

    refresh_app_data_conditionally(
        &[AppDataRefreshScope::Dolls],
        is_active.then_some(&[AppDataRefreshScope::User, AppDataRefreshScope::Friends]),
    ).await;

    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_doll(id: String) -> Result<(), String> {
    DollsRemote::new()
        .delete_doll(&id)
        .await
        .map_err(|e| e.to_string())?;

    // Check if this was the active doll (after delete completes to avoid stale reads)
    let is_active = is_active_doll(&id);

    refresh_app_data_conditionally(
        &[AppDataRefreshScope::Dolls],
        is_active.then_some(&[AppDataRefreshScope::User, AppDataRefreshScope::Friends]),
    ).await;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn set_active_doll(doll_id: String) -> Result<(), String> {
    UserRemote::new()
        .set_active_doll(&doll_id)
        .await
        .map_err(|e| e.to_string())?;

    refresh_app_data(&[AppDataRefreshScope::User, AppDataRefreshScope::Friends]).await;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn remove_active_doll() -> Result<(), String> {
    UserRemote::new()
        .remove_active_doll()
        .await
        .map_err(|e| e.to_string())?;

    refresh_app_data(&[AppDataRefreshScope::User, AppDataRefreshScope::Friends]).await;

    Ok(())
}
