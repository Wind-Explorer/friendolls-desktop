use tauri::Manager;
use tauri_plugin_positioner::WindowExt;

use crate::{
    get_app_handle,
    services::overlay::{overlay_fullscreen, SCENE_WINDOW_LABEL},
};

pub async fn start_fdoll() {
    initialize_session().await;
}

pub async fn initialize_session() {
    let webview_window = tauri::WebviewWindowBuilder::new(
        get_app_handle(),
        SCENE_WINDOW_LABEL,
        tauri::WebviewUrl::App("/scene".into()),
    )
    .title("Friendolls Scene")
    .inner_size(600.0, 500.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .visible(true)
    .skip_taskbar(true)
    .always_on_top(true)
    .visible_on_all_workspaces(true)
    .build()
    .expect("Failed to display scene screen");

    webview_window
        .move_window(tauri_plugin_positioner::Position::Center)
        .unwrap();

    let window = get_app_handle().get_window(webview_window.label()).unwrap();
    overlay_fullscreen(&window).unwrap();
    window.set_ignore_cursor_events(true).unwrap();

    println!("Scene window initialized.");
}
