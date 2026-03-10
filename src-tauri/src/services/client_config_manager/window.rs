use tauri::Manager;
use tracing::error;

use crate::get_app_handle;

use super::{ClientConfigError, CLIENT_CONFIG_MANAGER_WINDOW_LABEL};

#[tauri::command]
pub fn open_config_manager_window() -> Result<(), ClientConfigError> {
    let app_handle = get_app_handle();
    let existing_webview_window = app_handle.get_window(CLIENT_CONFIG_MANAGER_WINDOW_LABEL);

    if let Some(window) = existing_webview_window {
        if let Err(e) = window.show() {
            error!("Failed to show client config manager window: {e}");
            return Err(ClientConfigError::ShowWindow(e));
        }
        if let Err(e) = window.set_focus() {
            error!("Failed to focus client config manager window: {e}");
        }
        return Ok(());
    }

    match tauri::WebviewWindowBuilder::new(
        app_handle,
        CLIENT_CONFIG_MANAGER_WINDOW_LABEL,
        tauri::WebviewUrl::App("/client-config-manager".into()),
    )
    .title("Advanced Configuration")
    .inner_size(300.0, 420.0)
    .resizable(false)
    .maximizable(false)
    .visible(false)
    .build()
    {
        Ok(window) => {
            if let Err(e) = window.show() {
                error!("Failed to show client config manager window: {}", e);
                return Err(ClientConfigError::ShowWindow(e));
            }
            if let Err(e) = window.set_focus() {
                error!("Failed to focus client config manager window: {e}");
            }
            Ok(())
        }
        Err(e) => {
            error!("Failed to build client config manager window: {}", e);
            Err(ClientConfigError::Window(e))
        }
    }
}
