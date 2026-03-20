use tauri::Manager;
use tauri_plugin_positioner::WindowExt;
use tracing::{error, info};

use crate::get_app_handle;
use crate::services::window_manager::{
    ensure_window, EnsureWindowError, EnsureWindowResult, WindowConfig,
};

use super::interactivity::start_scene_modifier_listener;

pub static SCENE_WINDOW_LABEL: &str = "scene";
pub static SPLASH_WINDOW_LABEL: &str = "splash";

pub fn overlay_fullscreen(window: &tauri::Window) -> Result<(), tauri::Error> {
    let monitor = get_app_handle().primary_monitor()?.unwrap();
    let monitor_size = monitor.size();

    window.set_size(tauri::PhysicalSize {
        width: monitor_size.width,
        height: monitor_size.height,
    })?;

    window.set_position(tauri::PhysicalPosition { x: 0, y: 0 })?;

    Ok(())
}

pub fn open_splash_window() {
    info!("Starting splash window creation...");

    let mut config =
        WindowConfig::accessory(SPLASH_WINDOW_LABEL, "/splash.html", "Friendolls Splash");
    config.width = 800.0;
    config.height = 400.0;
    config.visible = false;

    let webview_window = match ensure_window(&config, true, false) {
        Ok(EnsureWindowResult::Created(window)) => window,
        Ok(EnsureWindowResult::Existing(_)) => return,
        Err(EnsureWindowError::MissingParent(parent_label)) => {
            error!(
                "Failed to build splash window due to missing parent '{}': impossible state",
                parent_label
            );
            return;
        }
        Err(EnsureWindowError::ShowExisting(e))
        | Err(EnsureWindowError::SetParent(e))
        | Err(EnsureWindowError::Build(e)) => {
            error!("Failed to build splash window: {}", e);
            return;
        }
    };

    if let Err(e) = webview_window.move_window(tauri_plugin_positioner::Position::Center) {
        error!("Failed to move splash window to center: {}", e);
    }

    if let Err(e) = webview_window.show() {
        error!("Failed to show splash window: {}", e);
    }

    info!("Splash window initialized successfully.");
}

pub fn close_splash_window() {
    let app_handle = get_app_handle();
    if let Some(window) = app_handle.get_window(SPLASH_WINDOW_LABEL) {
        if let Err(e) = window.close() {
            error!("Failed to close splash window: {}", e);
        } else {
            info!("Splash window closed");
        }
    }
}

pub fn open_scene_window() {
    info!("Starting scene creation...");

    let mut config = WindowConfig::accessory(SCENE_WINDOW_LABEL, "/scene", "Friendolls Scene");
    config.width = 600.0;
    config.height = 500.0;
    config.visible_on_all_workspaces = true;

    let webview_window = match ensure_window(&config, true, false) {
        Ok(EnsureWindowResult::Created(window)) => window,
        Ok(EnsureWindowResult::Existing(_)) => return,
        Err(EnsureWindowError::MissingParent(parent_label)) => {
            error!(
                "Failed to build scene window due to missing parent '{}': impossible state",
                parent_label
            );
            return;
        }
        Err(EnsureWindowError::ShowExisting(e))
        | Err(EnsureWindowError::SetParent(e))
        | Err(EnsureWindowError::Build(e)) => {
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

    start_scene_modifier_listener();

    #[cfg(debug_assertions)]
    webview_window.open_devtools();

    info!("Scene window initialized successfully.");
}
