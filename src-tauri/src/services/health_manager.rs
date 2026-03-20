use crate::get_app_handle;
use crate::{lock_r, state::FDOLL, system_tray::update_system_tray};
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder, MessageDialogKind};
use tauri_plugin_positioner::WindowExt;
use tracing::{error, info};

use super::window_manager::{
    encode_query_value, ensure_window, EnsureWindowError, EnsureWindowResult, WindowConfig,
};

pub static HEALTH_MANAGER_WINDOW_LABEL: &str = "health_manager";

/// Closes primary UI windows and shows the health manager with an optional error message.
pub fn open_health_manager_window(error_message: Option<String>) {
    let app_handle = get_app_handle();

    info!("Building health manager window");
    let mut config = WindowConfig::regular_ui(
        HEALTH_MANAGER_WINDOW_LABEL,
        format!(
            "/health-manager?err={}",
            encode_query_value(error_message.as_deref().unwrap_or("Something went wrong!"))
        ),
        "Health Manager",
    );
    config.visible = false;

    let webview_window = match ensure_window(&config, true, false) {
        Ok(EnsureWindowResult::Created(window)) => window,
        Ok(EnsureWindowResult::Existing(_)) => return,
        Err(EnsureWindowError::MissingParent(parent_label)) => {
            error!(
                "Failed to build {} window due to missing parent '{}': impossible state",
                HEALTH_MANAGER_WINDOW_LABEL, parent_label
            );
            return;
        }
        Err(EnsureWindowError::ShowExisting(e)) => {
            error!("Failed to show existing health manager window: {}", e);
            MessageDialogBuilder::new(
                app_handle.dialog().clone(),
                "Window Error",
                "Failed to show the health manager screen. Please restart and try again.",
            )
            .kind(MessageDialogKind::Error)
            .show(|_| {});
            return;
        }
        Err(EnsureWindowError::SetParent(e)) | Err(EnsureWindowError::Build(e)) => {
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

    if let Err(e) = webview_window.show() {
        error!("Failed to show health manager window: {}", e);
        MessageDialogBuilder::new(
            app_handle.dialog().clone(),
            "Window Error",
            "Failed to show the health manager screen. Please restart and try again.",
        )
        .kind(MessageDialogKind::Error)
        .show(|_| {});
    } else {
        info!("Health manager window shown successfully");
    }
}

pub fn close_health_manager_window() {
    let app_handle = get_app_handle();
    if let Some(window) = app_handle.get_window(HEALTH_MANAGER_WINDOW_LABEL) {
        if let Err(e) = window.close() {
            error!("Failed to close health manager window: {}", e);
        } else {
            info!("Health manager window closed");
            let guard = lock_r!(FDOLL);
            let is_logged_in = guard.user_data.user.is_some();
            drop(guard);
            update_system_tray(is_logged_in);
        }
    }
}
