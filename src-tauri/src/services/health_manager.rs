use crate::get_app_handle;
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder, MessageDialogKind};
use tauri_plugin_positioner::WindowExt;
use tracing::{error, info};

pub static HEALTH_MANAGER_WINDOW_LABEL: &str = "health_manager";
pub static HEALTH_MANAGER_EVENT: &str = "health-error";

fn close_window_if_exists(label: &str) {
    let app_handle = get_app_handle();
    if let Some(window) = app_handle.get_window(label) {
        if let Err(e) = window.close() {
            error!("Failed to close {} window: {}", label, e);
        }
    }
}

/// Closes primary UI windows and shows the health manager with an optional error message.
pub fn show_health_manager_with_error(error_message: Option<String>) {
    let app_handle = get_app_handle();
    // Ensure other windows are closed before showing health manager
    close_window_if_exists(crate::services::scene::SPLASH_WINDOW_LABEL);
    close_window_if_exists(crate::services::scene::SCENE_WINDOW_LABEL);
    close_window_if_exists(crate::services::app_menu::APP_MENU_WINDOW_LABEL);

    let existing_webview_window = app_handle.get_window(HEALTH_MANAGER_WINDOW_LABEL);

    if let Some(window) = existing_webview_window {
        if let Err(e) = window.show() {
            error!("Failed to show existing health manager window: {}", e);
            MessageDialogBuilder::new(
                app_handle.dialog().clone(),
                "Window Error",
                "Failed to show the health manager screen. Please restart and try again.",
            )
            .kind(MessageDialogKind::Error)
            .show(|_| {});
        }

        if let Some(message) = error_message {
            if let Err(e) = window.emit(HEALTH_MANAGER_EVENT, message.clone()) {
                error!("Failed to emit health error event: {}", e);
            }
        }
        return;
    }

    let webview_window = match tauri::WebviewWindowBuilder::new(
        app_handle,
        HEALTH_MANAGER_WINDOW_LABEL,
        tauri::WebviewUrl::App("/health-manager".into()),
    )
    .title("Health Manager")
    .inner_size(420.0, 420.0)
    .resizable(false)
    .decorations(true)
    .transparent(false)
    .shadow(true)
    .visible(false)
    .skip_taskbar(false)
    .always_on_top(false)
    .visible_on_all_workspaces(false)
    .build()
    {
        Ok(window) => {
            info!("{} window builder succeeded", HEALTH_MANAGER_WINDOW_LABEL);
            window
        }
        Err(e) => {
            error!(
                "Failed to build {} window: {}",
                HEALTH_MANAGER_WINDOW_LABEL, e
            );
            return;
        }
    };

    if let Err(e) = webview_window.move_window(tauri_plugin_positioner::Position::Center) {
        error!("Failed to move health manager window to center: {}", e);
    }

    if let Some(message) = error_message {
        if let Err(e) = webview_window.emit(HEALTH_MANAGER_EVENT, message.clone()) {
            error!("Failed to emit health error event: {}", e);
        }
    }

    if let Err(e) = webview_window.show() {
        error!("Failed to show health manager window: {}", e);
        MessageDialogBuilder::new(
            app_handle.dialog().clone(),
            "Window Error",
            "Failed to show the health manager screen. Please restart and try again.",
        )
        .kind(MessageDialogKind::Error)
        .show(|_| {});
    }
}

pub fn close_health_manager_window() {
    let app_handle = get_app_handle();
    if let Some(window) = app_handle.get_window(HEALTH_MANAGER_WINDOW_LABEL) {
        if let Err(e) = window.close() {
            error!("Failed to close health manager window: {}", e);
        } else {
            info!("Health manager window closed");
        }
    }
}
