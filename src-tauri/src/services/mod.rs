use tauri::Manager;

use crate::get_app_handle;

pub mod active_app;
pub mod app_menu;
pub mod auth;
pub mod client_config_manager;
pub mod cursor;
pub mod doll_editor;
pub mod health_manager;
pub mod interaction;
pub mod scene;
pub mod sprite_recolor;
pub mod welcome;
pub mod ws;

pub fn close_all_windows() {
    let app_handle = get_app_handle();
    let webview_windows = app_handle.webview_windows();
    for window in webview_windows {
        window.1.close().unwrap();
    }
}
