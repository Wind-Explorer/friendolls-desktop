use crate::{
    models::app_data::AppData,
    remotes::friends::{
        FriendRemote, FriendRequestResponseDto, FriendshipResponseDto, SendFriendRequestDto,
        UserBasicDto,
    },
    services::cursor::start_cursor_tracking,
    state::{init_app_data, FDOLL},
};
use tauri::async_runtime;
use tauri::Manager;
use tracing_subscriber;

static APP_HANDLE: std::sync::OnceLock<tauri::AppHandle<tauri::Wry>> = std::sync::OnceLock::new();

mod app;
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

    // Keep the guard alive?
    // Actually `_guard` will be dropped here, which might stop logging.
    // Ideally we should store the guard in the app state or use a global lazy_static if we want it to persist.
    // However, `tracing_appender` docs say: "WorkerGuard should be assigned in the main function or whatever the entrypoint of the program is."
    // Since we are inside `setup_fdoll` which is called from `setup`, we might lose logs if we drop it.
    // But for simplicity in this context, we can just let it leak or store it in a static.
    // Let's leak it for now as this is a long-running app.
    Box::leak(Box::new(_guard));

    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_writer(non_blocking) // Log to file
        .init();

    state::init_fdoll_state();
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

#[tauri::command]
fn get_app_data() -> Result<AppData, String> {
    let guard = lock_r!(FDOLL);
    return Ok(guard.app_data.clone());
}

#[tauri::command]
async fn refresh_app_data() -> Result<AppData, String> {
    init_app_data().await;
    let guard = lock_r!(FDOLL);
    Ok(guard.app_data.clone())
}

#[tauri::command]
async fn list_friends() -> Result<Vec<FriendshipResponseDto>, String> {
    FriendRemote::new()
        .get_friends()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_users(username: Option<String>) -> Result<Vec<UserBasicDto>, String> {
    tracing::info!(
        "Tauri command search_users called with username: {:?}",
        username
    );
    let remote = FriendRemote::new();
    tracing::info!("FriendRemote instance created for search_users");
    let result = remote.search_users(username.as_deref()).await;
    tracing::info!("FriendRemote::search_users result: {:?}", result);
    result.map_err(|e| {
        tracing::error!("search_users command error: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn send_friend_request(
    request: SendFriendRequestDto,
) -> Result<FriendRequestResponseDto, String> {
    FriendRemote::new()
        .send_friend_request(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn received_friend_requests() -> Result<Vec<FriendRequestResponseDto>, String> {
    FriendRemote::new()
        .get_received_requests()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn sent_friend_requests() -> Result<Vec<FriendRequestResponseDto>, String> {
    FriendRemote::new()
        .get_sent_requests()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn accept_friend_request(request_id: String) -> Result<FriendRequestResponseDto, String> {
    FriendRemote::new()
        .accept_friend_request(&request_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn deny_friend_request(request_id: String) -> Result<FriendRequestResponseDto, String> {
    FriendRemote::new()
        .deny_friend_request(&request_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn unfriend(friend_id: String) -> Result<(), String> {
    FriendRemote::new()
        .unfriend(&friend_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn quit_app() -> Result<(), String> {
    let app_handle = get_app_handle();
    app_handle.exit(0);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_dialog::init())
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
            quit_app
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
