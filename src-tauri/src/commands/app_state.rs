use crate::{
    lock_r,
    models::{app_data::UserData, dolls::DollColorSchemeDto},
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

#[tauri::command]
#[specta::specta]
pub fn get_active_doll_color_scheme() -> Result<Option<DollColorSchemeDto>, String> {
    let guard = lock_r!(FDOLL);
    let active_doll_id = guard
        .user_data
        .user
        .as_ref()
        .and_then(|u| u.active_doll_id.as_deref());

    match active_doll_id {
        Some(active_doll_id) => {
            let color_scheme = guard
                .user_data
                .dolls
                .as_ref()
                .and_then(|dolls| dolls.iter().find(|d| d.id == active_doll_id))
                .map(|d| d.configuration.color_scheme.clone());
            Ok(color_scheme)
        }
        None => Ok(None),
    }
}
