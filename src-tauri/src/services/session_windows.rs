use tauri::Manager;
use tracing::warn;

use crate::get_app_handle;

pub fn close_all_windows() {
    let app_handle = get_app_handle();
    for (label, window) in app_handle.webview_windows() {
        if let Err(error) = window.close() {
            warn!("Failed to close window '{}': {}", label, error);
        }
    }
}
