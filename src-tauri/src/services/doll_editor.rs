use tauri::{Emitter, Manager};
use tracing::{error, info};

use crate::get_app_handle;

static DOLL_EDITOR_WINDOW_LABEL: &str = "doll_editor";
static APP_MENU_WINDOW_LABEL: &str = "app_menu";

#[tauri::command]
pub async fn open_doll_editor_window(doll_id: Option<String>) {
    let app_handle = get_app_handle().clone();

    // Dispatch to main thread to avoid potential deadlocks on Windows when setting parent window
    let _ = app_handle.run_on_main_thread(move || {
        let app_handle = get_app_handle();

        // Check if the window already exists
        let existing_window = app_handle.get_webview_window(DOLL_EDITOR_WINDOW_LABEL);
        if let Some(window) = existing_window {
            // If it exists, we might want to reload it with new params or just focus it
            if let Err(e) = window.set_focus() {
                error!("Failed to focus existing doll editor window: {}", e);
            }

            // Emit event to update context
            if let Some(id) = doll_id {
                if let Err(e) = window.emit("edit-doll", id) {
                    error!("Failed to emit edit-doll event: {}", e);
                }
            } else {
                if let Err(e) = window.emit("create-doll", ()) {
                    error!("Failed to emit create-doll event: {}", e);
                }
            }

            return;
        }

        let url_path = if let Some(id) = doll_id {
            format!("/app-menu/tabs/your-dolls/doll-editor?id={}", id)
        } else {
            "/app-menu/tabs/your-dolls/doll-editor".to_string()
        };

        let mut builder = tauri::WebviewWindowBuilder::new(
            app_handle,
            DOLL_EDITOR_WINDOW_LABEL,
            tauri::WebviewUrl::App(url_path.into()),
        )
        .title("Doll Editor")
        .inner_size(300.0, 400.0)
        .resizable(false)
        .decorations(true)
        .transparent(true)
        .shadow(true)
        .visible(true)
        .skip_taskbar(false)
        .always_on_top(true) // Helper window, nice to stay on top
        .visible_on_all_workspaces(false);

        // Set parent if app menu exists
        if let Some(parent) = app_handle.get_webview_window(APP_MENU_WINDOW_LABEL) {
            match builder.parent(&parent) {
                Ok(b) => builder = b,
                Err(e) => {
                    error!("Failed to set parent for doll editor window: {}", e);
                    // If we fail to set parent, we effectively lost the builder because .parent() consumes it.
                    // We must return here to avoid using moved value.
                    return;
                }
            };
        }

        match builder.build() {
            Ok(window) => {
                info!("{} window builder succeeded", DOLL_EDITOR_WINDOW_LABEL);
                #[cfg(debug_assertions)]
                window.open_devtools();
            }
            Err(e) => {
                error!("Failed to build {} window: {}", DOLL_EDITOR_WINDOW_LABEL, e);
            }
        };
    });
}
