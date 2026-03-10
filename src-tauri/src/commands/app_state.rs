use crate::{
    lock_r,
    models::app_data::UserData,
    services::{
        friend_active_doll_sprite, presence_modules::models::ModuleMetadata, sprite,
    },
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
pub fn get_active_doll_sprite_base64() -> Result<Option<String>, String> {
    sprite::get_active_doll_sprite_base64()
}

#[tauri::command]
#[specta::specta]
pub fn get_friend_active_doll_sprites_base64(
) -> Result<friend_active_doll_sprite::FriendActiveDollSpritesDto, String> {
    friend_active_doll_sprite::sync_from_app_data();
    Ok(friend_active_doll_sprite::get_snapshot())
}
