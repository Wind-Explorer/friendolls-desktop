use tauri::Manager;
use tracing::{error, info};

use crate::get_app_handle;

pub static APP_MENU_WINDOW_LABEL: &str = "app_menu";

pub fn open_app_menu_window() {
    let app_handle = get_app_handle();
    let existing_webview_window = app_handle.get_window(APP_MENU_WINDOW_LABEL);

    if let Some(window) = existing_webview_window {
        window.show().unwrap();
        return;
    }

    match tauri::WebviewWindowBuilder::new(
        app_handle,
        APP_MENU_WINDOW_LABEL,
        tauri::WebviewUrl::App("/app-menu".into()),
    )
    .title("Friendolls")
    .inner_size(400.0, 550.0)
    .resizable(true)
    .maximizable(false)
    .decorations(true)
    .transparent(false)
    .shadow(true)
    .visible(true)
    .skip_taskbar(false)
    .always_on_top(false)
    .visible_on_all_workspaces(false)
    .build()
    {
        Ok(window) => {
            info!("{} window builder succeeded", APP_MENU_WINDOW_LABEL);
            window
        }
        Err(e) => {
            error!("Failed to build {} window: {}", APP_MENU_WINDOW_LABEL, e);
            return;
        }
    };
}
