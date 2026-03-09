use crate::{
    commands::app_state::get_modules,
    services::{
        doll_editor::open_doll_editor_window,
        scene::{get_scene_interactive, set_pet_menu_state, set_scene_interactive},
    },
};
use commands::app::{quit_app, restart_app, retry_connection};
use commands::app_state::{get_app_data, get_active_doll_color_scheme, refresh_app_data};
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
use commands::petpet::encode_pet_doll_gif_base64;
use specta_typescript::Typescript;
use tauri::async_runtime;
use tauri_specta::{Builder as SpectaBuilder, ErrorHandlingMode, collect_commands, collect_events};

use crate::services::app_events::{
    AppDataRefreshed, CreateDoll, CursorMoved, EditDoll, FriendActiveDollChanged,
    FriendCursorPositionsUpdated, FriendDisconnected,
    FriendRequestAccepted, FriendRequestDenied, FriendRequestReceived,
    FriendUserStatusChanged, InteractionDeliveryFailed, InteractionReceived,
    SceneInteractiveChanged, SetInteractionOverlay, Unfriended, UserStatusChanged,
};

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
    let specta_builder = SpectaBuilder::<tauri::Wry>::new()
        .error_handling(ErrorHandlingMode::Throw)
        .commands(collect_commands![
            get_app_data,
            get_active_doll_color_scheme,
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
            encode_pet_doll_gif_base64,
            quit_app,
            restart_app,
            retry_connection,
            get_client_config,
            save_client_config,
            open_client_config_manager,
            open_doll_editor_window,
            get_scene_interactive,
            set_scene_interactive,
            set_pet_menu_state,
            login,
            register,
            change_password,
            reset_password,
            logout_and_restart,
            send_interaction_cmd,
            get_modules
        ])
        .events(collect_events![
            CursorMoved,
            SceneInteractiveChanged,
            AppDataRefreshed,
            SetInteractionOverlay,
            EditDoll,
            CreateDoll,
            UserStatusChanged,
            FriendCursorPositionsUpdated,
            FriendDisconnected,
            FriendActiveDollChanged,
            FriendUserStatusChanged,
            InteractionReceived,
            InteractionDeliveryFailed,
            FriendRequestReceived,
            FriendRequestAccepted,
            FriendRequestDenied,
            Unfriended
        ]);

    #[cfg(debug_assertions)]
    specta_builder
        .export(Typescript::default(), "../src/lib/bindings.ts")
        .expect("Failed to export TypeScript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);
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
