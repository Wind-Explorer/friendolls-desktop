use crate::{
    lock_r,
    models::{app_data::UserData, app_state::{AppState, NekoPosition}},
    services::{
        app_data::{init_app_data_scoped, AppDataRefreshScope},
        app_state,
        friends,
        neko_positions,
        presence_modules::models::ModuleMetadata,
        sprite,
    },
    state::FDOLL,
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
pub fn get_active_doll_sprite_base64() -> Result<Option<String>, String> {
    sprite::get_active_doll_sprite_base64()
}

#[tauri::command]
#[specta::specta]
pub fn get_friend_active_doll_sprites_base64() -> Result<friends::FriendActiveDollSpritesDto, String>
{
    friends::sync_active_doll_sprites_from_app_data();
    Ok(friends::get_active_doll_sprites_snapshot())
}

#[tauri::command]
#[specta::specta]
pub fn get_app_state() -> Result<AppState, String> {
    Ok(app_state::get_snapshot())
}

#[tauri::command]
#[specta::specta]
pub fn get_neko_positions() -> Result<neko_positions::NekoPositionsDto, String> {
    Ok(neko_positions::get_snapshot())
}

#[tauri::command]
#[specta::specta]
pub fn set_scene_setup_nekos_position(nekos_position: Option<NekoPosition>) {
    app_state::set_scene_setup_nekos_position(nekos_position);
}

#[tauri::command]
#[specta::specta]
pub fn set_scene_setup_nekos_opacity(nekos_opacity: f32) {
    app_state::set_scene_setup_nekos_opacity(nekos_opacity);
}

#[tauri::command]
#[specta::specta]
pub fn set_scene_setup_nekos_scale(nekos_scale: f32) {
    app_state::set_scene_setup_nekos_scale(nekos_scale);
}
