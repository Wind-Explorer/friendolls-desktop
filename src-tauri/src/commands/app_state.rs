use crate::{
    lock_r,
    models::app_data::UserData,
    services::presence_modules::models::ModuleMetadata,
    state::{init_app_data_scoped, AppDataRefreshScope, FDOLL},
};

#[tauri::command]
#[specta::specta]
pub fn get_app_data() -> Result<UserData, String> {
    let guard = lock_r!(FDOLL);
    Ok(guard.user_data.clone())
}

#[tauri::command]
#[specta::specta]
pub async fn refresh_app_data() -> Result<UserData, String> {
    init_app_data_scoped(AppDataRefreshScope::All).await;
    let guard = lock_r!(FDOLL);
    Ok(guard.user_data.clone())
}

#[tauri::command]
#[specta::specta]
pub fn get_modules() -> Result<Vec<ModuleMetadata>, String> {
    let guard = lock_r!(FDOLL);
    Ok(guard.modules.metadatas.clone())
}
