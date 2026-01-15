use crate::{
    lock_r,
    models::app_data::AppData,
    state::{init_app_data, FDOLL},
};

#[tauri::command]
pub fn get_app_data() -> Result<AppData, String> {
    let guard = lock_r!(FDOLL);
    Ok(guard.app_data.clone())
}

#[tauri::command]
pub async fn refresh_app_data() -> Result<AppData, String> {
    init_app_data().await;
    let guard = lock_r!(FDOLL);
    Ok(guard.app_data.clone())
}
