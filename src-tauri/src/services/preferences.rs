use tracing::{error, info};

use crate::get_app_handle;

pub fn create_preferences_window() {
    let webview_window = match tauri::WebviewWindowBuilder::new(
        get_app_handle(),
        "preferences",
        tauri::WebviewUrl::App("/preferences".into()),
    )
    .title("Friendolls Preferences")
    .inner_size(600.0, 500.0)
    .resizable(true)
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
            info!("Preferences window builder succeeded");
            window
        }
        Err(e) => {
            error!("Failed to build Preferences window: {}", e);
            return;
        }
    };

    #[cfg(debug_assertions)]
    webview_window.open_devtools();
}
