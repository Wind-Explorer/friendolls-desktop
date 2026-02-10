use crate::services::{
    doll_editor::open_doll_editor_window,
    scene::{set_pet_menu_state, set_scene_interactive},
};
use commands::app::{quit_app, restart_app, retry_connection};
use commands::app_data::{get_app_data, refresh_app_data};
use commands::auth::{change_password, login, logout_and_restart, register, reset_password};
use commands::config::{get_client_config, open_client_config_manager, save_client_config};
use commands::dolls::{
    create_doll, delete_doll, get_doll, get_dolls, remove_active_doll, set_active_doll, update_doll,
};
use commands::friends::{
    accept_friend_request, deny_friend_request, list_friends, received_friend_requests,
    search_users, send_friend_request, sent_friend_requests, unfriend,
};
use commands::interaction::send_interaction_cmd;
use commands::sprite::recolor_gif_base64;
use commands::user_status::send_user_status_cmd;
use tauri::async_runtime;

static APP_HANDLE: std::sync::OnceLock<tauri::AppHandle<tauri::Wry>> = std::sync::OnceLock::new();

mod commands;
mod init;
mod models;
mod remotes;
mod services;
mod state;
mod system_tray;
mod utilities;

/// Tauri app handle
pub fn get_app_handle<'a>() -> &'a tauri::AppHandle<tauri::Wry> {
    APP_HANDLE
        .get()
        .expect("get_app_handle called but app is still not initialized")
}

fn register_app_events(event: tauri::RunEvent) {
    if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
        if code.is_none() {
            api.prevent_exit();
        } else {
            println!("exit code: {:?}", code);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            get_app_data,
            refresh_app_data,
            list_friends,
            search_users,
            send_friend_request,
            received_friend_requests,
            sent_friend_requests,
            accept_friend_request,
            deny_friend_request,
            unfriend,
            get_dolls,
            get_doll,
            create_doll,
            update_doll,
            delete_doll,
            set_active_doll,
            remove_active_doll,
            recolor_gif_base64,
            quit_app,
            restart_app,
            retry_connection,
            get_client_config,
            save_client_config,
            open_client_config_manager,
            open_doll_editor_window,
            set_scene_interactive,
            set_pet_menu_state,
            login,
            register,
            change_password,
            reset_password,
            logout_and_restart,
            send_interaction_cmd,
            send_user_status_cmd
        ])
        .setup(|app| {
            APP_HANDLE
                .set(app.handle().to_owned())
                .expect("Failed to init app handle.");
            async_runtime::spawn(async move { init::launch_app().await });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_, event| register_app_events(event));
}
