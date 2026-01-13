use crate::get_app_handle;
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder, MessageDialogKind};
use tauri_plugin_positioner::WindowExt;
use tracing::{error, info};

pub static WELCOME_WINDOW_LABEL: &str = "welcome";

pub fn open_welcome_window() {
    let app_handle = get_app_handle();
    let existing_webview_window = app_handle.get_window(WELCOME_WINDOW_LABEL);

    if let Some(window) = existing_webview_window {
        if let Err(e) = window.show() {
            error!("Failed to show existing welcome window: {}", e);
            MessageDialogBuilder::new(
                app_handle.dialog().clone(),
                "Window Error",
                "Failed to show the welcome screen. Please restart and try again.",
            )
            .kind(MessageDialogKind::Error)
            .show(|_| {});
        }
        return;
    }

    let webview_window = match tauri::WebviewWindowBuilder::new(
        app_handle,
        WELCOME_WINDOW_LABEL,
        tauri::WebviewUrl::App("/welcome".into()),
    )
    .title("Welcome to Friendolls")
    .inner_size(420.0, 420.0)
    .resizable(false)
    .maximizable(false)
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
            info!("{} window builder succeeded", WELCOME_WINDOW_LABEL);
            window
        }
        Err(e) => {
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
