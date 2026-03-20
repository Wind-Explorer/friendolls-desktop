use crate::get_app_handle;
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder, MessageDialogKind};
use tauri_plugin_positioner::WindowExt;
use tracing::{error, info};

use super::window_manager::{ensure_window, EnsureWindowError, EnsureWindowResult, WindowConfig};

pub static WELCOME_WINDOW_LABEL: &str = "welcome";

pub fn open_welcome_window() {
    let app_handle = get_app_handle();

    let mut config =
        WindowConfig::regular_ui(WELCOME_WINDOW_LABEL, "/welcome", "Welcome to Friendolls");
    config.visible = false;

    let webview_window = match ensure_window(&config, true, false) {
        Ok(EnsureWindowResult::Created(window)) => window,
        Ok(EnsureWindowResult::Existing(_)) => return,
        Err(EnsureWindowError::MissingParent(parent_label)) => {
            error!(
                "Failed to build {} window due to missing parent '{}': impossible state",
                WELCOME_WINDOW_LABEL, parent_label
            );
            return;
        }
        Err(EnsureWindowError::ShowExisting(e)) => {
            error!("Failed to show existing welcome window: {}", e);
            MessageDialogBuilder::new(
                app_handle.dialog().clone(),
                "Window Error",
                "Failed to show the welcome screen. Please restart and try again.",
            )
            .kind(MessageDialogKind::Error)
            .show(|_| {});
            return;
        }
        Err(EnsureWindowError::SetParent(e)) | Err(EnsureWindowError::Build(e)) => {
            error!("Failed to build {} window: {}", WELCOME_WINDOW_LABEL, e);
            return;
        }
    };

    if let Err(e) = webview_window.move_window(tauri_plugin_positioner::Position::Center) {
        error!("Failed to move welcome window to center: {}", e);
    }

    if let Err(e) = webview_window.show() {
        error!("Failed to show welcome window: {}", e);
        MessageDialogBuilder::new(
            app_handle.dialog().clone(),
            "Window Error",
            "Failed to show the welcome screen. Please restart and try again.",
        )
        .kind(MessageDialogKind::Error)
        .show(|_| {});
    }
}

pub fn close_welcome_window() {
    let app_handle = get_app_handle();
    if let Some(window) = app_handle.get_window(WELCOME_WINDOW_LABEL) {
        if let Err(e) = window.close() {
            error!("Failed to close welcome window: {}", e);
        } else {
            info!("Welcome window closed");
        }
    }
}
