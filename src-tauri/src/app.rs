use tauri::Manager;
use tauri_plugin_positioner::WindowExt;
use tracing::{error, info};

use crate::{
    get_app_handle,
    services::{
        auth::get_tokens,
        overlay::{overlay_fullscreen, SCENE_WINDOW_LABEL},
        preferences::create_preferences_window,
    },
    state::init_app_data,
};

pub async fn start_fdoll() {
    bootstrap().await;
}

async fn construct_app() {
    init_app_data().await;
    create_scene();
    create_preferences_window();
}

pub async fn bootstrap() {
    match get_tokens().await {
        Some(_) => {
            info!("User session restored");
            construct_app().await;
        }
        None => {
            info!("No active session, user needs to authenticate");
            crate::services::auth::init_auth_code_retrieval(|| {
                info!("Authentication successful, creating scene...");
                tauri::async_runtime::spawn(async {
                    info!("Creating scene after auth success...");
                    construct_app().await;
                });
            });
        }
    }
}

pub fn create_scene() {
    info!("Starting scene creation...");
    let webview_window = match tauri::WebviewWindowBuilder::new(
        get_app_handle(),
        SCENE_WINDOW_LABEL,
        tauri::WebviewUrl::App("/scene".into()),
    )
    .title("Friendolls Scene")
    .inner_size(600.0, 500.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .visible(true)
    .skip_taskbar(true)
    .always_on_top(true)
    .visible_on_all_workspaces(true)
    .build()
    {
        Ok(window) => {
            info!("Scene window builder succeeded");
            window
        }
        Err(e) => {
            error!("Failed to build scene window: {}", e);
            return;
        }
    };

    if let Err(e) = webview_window.move_window(tauri_plugin_positioner::Position::Center) {
        error!("Failed to move scene window to center: {}", e);
        return;
    }

    let window = match get_app_handle().get_window(webview_window.label()) {
        Some(window) => window,
        None => {
            error!("Failed to get scene window after creation");
            return;
        }
    };

    if let Err(e) = overlay_fullscreen(&window) {
        error!("Failed to set overlay fullscreen: {}", e);
        return;
    }

    if let Err(e) = window.set_ignore_cursor_events(true) {
        error!("Failed to set ignore cursor events: {}", e);
        return;
    }

    #[cfg(debug_assertions)]
    webview_window.open_devtools();

    info!("Scene window initialized successfully.");
}
