use crate::{
    lock_w,
    services::client_config_manager::{
        load_app_config, open_config_manager_window, save_app_config, AppConfig,
    },
    state::FDOLL,
};

#[tauri::command]
#[specta::specta]
pub fn get_client_config() -> AppConfig {
    let mut guard = lock_w!(FDOLL);
    guard.app_config = load_app_config();
    guard.app_config.clone()
}

#[tauri::command]
#[specta::specta]
pub fn save_client_config(config: AppConfig) -> Result<(), String> {
    match save_app_config(config) {
        Ok(saved) => {
            let mut guard = lock_w!(FDOLL);
            guard.app_config = saved;
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn open_client_config_manager() -> Result<(), String> {
    open_config_manager_window().map_err(|e| e.to_string())
}
