use crate::services::{
    cursor::start_cursor_tracking,
    doll_editor::open_doll_editor_window,
    scene::{open_splash_window, set_pet_menu_state, set_scene_interactive},
};
use commands::app::{quit_app, restart_app};
use commands::app_data::{get_app_data, refresh_app_data};
use commands::auth::{logout_and_restart, start_auth_flow};
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
use tauri::async_runtime;
use tauri::Manager;
use tracing_subscriber::{self, util::SubscriberInitExt};

static APP_HANDLE: std::sync::OnceLock<tauri::AppHandle<tauri::Wry>> = std::sync::OnceLock::new();

mod app;
mod commands;
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

fn setup_fdoll() -> Result<(), tauri::Error> {
    // Initialize tracing subscriber for logging

    // Set up file appender
    let app_handle = get_app_handle();
    let app_log_dir = app_handle
        .path()
        .app_log_dir()
        .expect("Could not determine app log dir");

    // Create the directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&app_log_dir) {
        eprintln!("Failed to create log directory: {}", e);
    }

    let file_appender = tracing_appender::rolling::daily(&app_log_dir, "friendolls.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Create a filter - adjust the level as needed (trace, debug, info, warn, error)
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    // Create a layer that writes to the file
    let file_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_writer(non_blocking);

    // Create a layer that writes to stdout (console)
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true);

    // Combine both layers with filter
    use tracing_subscriber::layer::SubscriberExt;
    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .init();

    open_splash_window();

    state::init_fdoll_state(Some(_guard));
    async_runtime::spawn(async move { app::start_fdoll().await });
    Ok(())
}

fn register_app_events(event: tauri::RunEvent) {
    match event {
        tauri::RunEvent::ExitRequested { api, code, .. } => {
            if code.is_none() {
                api.prevent_exit();
            } else {
                println!("exit code: {:?}", code);
            }
        }
        _ => {}
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
            start_cursor_tracking,
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
            get_client_config,
            save_client_config,
            open_client_config_manager,
            open_doll_editor_window,
            set_scene_interactive,
            set_pet_menu_state,
            start_auth_flow,
            logout_and_restart,
            send_interaction_cmd
        ])
        .setup(|app| {
            APP_HANDLE
                .set(app.handle().to_owned())
                .expect("Failed to init app handle.");
            setup_fdoll().expect("Failed to setup app.");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_, event| register_app_events(event));
}
