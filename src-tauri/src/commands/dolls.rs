use crate::{
    lock_r,
    models::dolls::{CreateDollDto, DollDto, UpdateDollDto},
    remotes::{
        dolls::DollsRemote,
        user::UserRemote,
    },
    state::{init_app_data_scoped, AppDataRefreshScope, FDOLL},
};
use tauri::async_runtime;

#[tauri::command]
pub async fn get_dolls() -> Result<Vec<DollDto>, String> {
    DollsRemote::new()
        .get_dolls()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_doll(id: String) -> Result<DollDto, String> {
    DollsRemote::new()
        .get_doll(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_doll(dto: CreateDollDto) -> Result<DollDto, String> {
    let result = DollsRemote::new()
        .create_doll(dto)
        .await
        .map_err(|e| e.to_string())?;

    // Refresh dolls list in background (deduped inside init_app_data_scoped)
    async_runtime::spawn(async {
        init_app_data_scoped(AppDataRefreshScope::Dolls).await;
    });

    Ok(result)
}

#[tauri::command]
pub async fn update_doll(id: String, dto: UpdateDollDto) -> Result<DollDto, String> {
    let result = DollsRemote::new()
        .update_doll(&id, dto)
        .await
        .map_err(|e| e.to_string())?;

    // Check if this was the active doll (after update completes to avoid stale reads)
    let is_active_doll = {
        let guard = lock_r!(FDOLL);
        guard
            .app_data
            .user
            .as_ref()
            .and_then(|u| u.active_doll_id.as_ref())
            .map(|active_id| active_id == &id)
            .unwrap_or(false)
    };

    // Refresh dolls list + User/Friends if this was the active doll
    async_runtime::spawn(async move {
        init_app_data_scoped(AppDataRefreshScope::Dolls).await;
        if is_active_doll {
            init_app_data_scoped(AppDataRefreshScope::User).await;
            init_app_data_scoped(AppDataRefreshScope::Friends).await;
        }
    });

    Ok(result)
}

#[tauri::command]
pub async fn delete_doll(id: String) -> Result<(), String> {
    DollsRemote::new()
        .delete_doll(&id)
        .await
        .map_err(|e| e.to_string())?;

    // Check if this was the active doll (after delete completes to avoid stale reads)
    let is_active_doll = {
        let guard = lock_r!(FDOLL);
        guard
            .app_data
            .user
            .as_ref()
            .and_then(|u| u.active_doll_id.as_ref())
            .map(|active_id| active_id == &id)
            .unwrap_or(false)
    };

    // Refresh dolls list + User/Friends if the deleted doll was active
    async_runtime::spawn(async move {
        init_app_data_scoped(AppDataRefreshScope::Dolls).await;
        if is_active_doll {
            init_app_data_scoped(AppDataRefreshScope::User).await;
            init_app_data_scoped(AppDataRefreshScope::Friends).await;
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn set_active_doll(doll_id: String) -> Result<(), String> {
    UserRemote::new()
        .set_active_doll(&doll_id)
        .await
        .map_err(|e| e.to_string())?;

    // Refresh User (for active_doll_id) + Friends (so friends see your active doll)
    // We don't need to refresh Dolls since the doll itself hasn't changed
    async_runtime::spawn(async {
        init_app_data_scoped(AppDataRefreshScope::User).await;
        init_app_data_scoped(AppDataRefreshScope::Friends).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn remove_active_doll() -> Result<(), String> {
    UserRemote::new()
        .remove_active_doll()
        .await
        .map_err(|e| e.to_string())?;

    // Refresh User (for active_doll_id) + Friends (so friends see your doll is gone)
    async_runtime::spawn(async {
        init_app_data_scoped(AppDataRefreshScope::User).await;
        init_app_data_scoped(AppDataRefreshScope::Friends).await;
    });

    Ok(())
}
