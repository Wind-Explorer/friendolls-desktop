use crate::{
    lock_r,
    models::app_data::AppData,
    state::{init_app_data_scoped, AppDataRefreshScope, FDOLL},
};

#[tauri::command]
pub fn get_app_data() -> Result<AppData, String> {
    let guard = lock_r!(FDOLL);
    Ok(guard.user_data.clone())
}

#[tauri::command]
pub async fn refresh_app_data() -> Result<AppData, String> {
    init_app_data_scoped(AppDataRefreshScope::All).await;
    let guard = lock_r!(FDOLL);
    Ok(guard.user_data.clone())
}
