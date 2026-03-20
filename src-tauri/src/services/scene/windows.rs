use tauri::Manager;
use tauri_plugin_positioner::WindowExt;
use tracing::{error, info};

#[cfg(target_os = "macos")]
use objc2_app_kit::{NSFloatingWindowLevel, NSWindow, NSWindowCollectionBehavior};

use crate::get_app_handle;
use crate::services::window_manager::{
    ensure_window, EnsureWindowError, EnsureWindowResult, WindowConfig,
};

use super::interactivity::start_scene_modifier_listener;

pub static SCENE_WINDOW_LABEL: &str = "scene";
pub static SPLASH_WINDOW_LABEL: &str = "splash";

pub fn overlay_fullscreen(window: &tauri::Window) -> Result<(), tauri::Error> {
    let monitor = get_app_handle().primary_monitor()?.unwrap();
    let monitor_position = monitor.position();
    let monitor_size = monitor.size();

    window.set_size(tauri::PhysicalSize {
        width: monitor_size.width,
        height: monitor_size.height,
    })?;

    window.set_position(*monitor_position)?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn apply_scene_window_macos_policy(window: &tauri::Window) -> Result<(), tauri::Error> {
    let ns_window = unsafe { &*window.ns_window()?.cast::<NSWindow>() };
    let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
        | NSWindowCollectionBehavior::Stationary
        | NSWindowCollectionBehavior::IgnoresCycle;

    ns_window.setLevel(NSFloatingWindowLevel);
    ns_window.setCollectionBehavior(behavior);

    Ok(())
}

#[cfg(target_os = "macos")]
fn harden_scene_window_on_macos(window: &tauri::Window) {
    let app_handle = get_app_handle().clone();
    let app_handle_for_closure = app_handle.clone();
    let window_label = window.label().to_string();

    if let Err(e) = app_handle.run_on_main_thread(move || {
        let Some(window) = app_handle_for_closure.get_window(&window_label) else {
            error!(
                "Failed to apply macOS scene hardening: window '{}' not found",
                window_label
            );
            return;
        };

        if let Err(e) = apply_scene_window_macos_policy(&window) {
            error!("Failed to apply macOS scene hardening policy: {}", e);
        }
    }) {
        error!("Failed to schedule macOS scene hardening policy: {}", e);
    }
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

    #[cfg(target_os = "macos")]
    harden_scene_window_on_macos(&window);

    if let Err(e) = window.set_ignore_cursor_events(true) {
        error!("Failed to set ignore cursor events: {}", e);
        return;
    }

    start_scene_modifier_listener();

    #[cfg(debug_assertions)]
    webview_window.open_devtools();

    info!("Scene window initialized successfully.");
}
