use crate::get_app_handle;
use tauri::Manager;
use tauri_plugin_positioner::WindowExt;
use tracing::{error, info};
pub static SCENE_WINDOW_LABEL: &str = "scene";

pub fn overlay_fullscreen(window: &tauri::Window) -> Result<(), tauri::Error> {
    // Get the primary monitor
    let monitor = get_app_handle().primary_monitor()?.unwrap();
    // Get the work area (usable space, excluding menu bar/dock/notch)
    let work_area = monitor.work_area();
    // Set window position to top-left of the work area
    window.set_position(tauri::PhysicalPosition {
        x: work_area.position.x,
        y: work_area.position.y,
    })?;
    // Set window size to match work area size
    window.set_size(tauri::PhysicalSize {
        width: work_area.size.width,
        height: work_area.size.height,
    })?;
    Ok(())
}

pub fn create_scene_window() {
    info!("Starting scene creation...");
    let webview_window = match tauri::WebviewWindowBuilder::new(
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
    {
        Ok(window) => {
            info!("Scene window builder succeeded");
            window
        }
        Err(e) => {
            error!("Failed to build scene window: {}", e);
            return;
        }
    };

    if let Err(e) = webview_window.move_window(tauri_plugin_positioner::Position::Center) {
        error!("Failed to move scene window to center: {}", e);
        return;
    }

    let window = match get_app_handle().get_window(webview_window.label()) {
        Some(window) => window,
        None => {
            error!("Failed to get scene window after creation");
            return;
        }
    };

    if let Err(e) = overlay_fullscreen(&window) {
        error!("Failed to set overlay fullscreen: {}", e);
        return;
    }

    if let Err(e) = window.set_ignore_cursor_events(true) {
        error!("Failed to set ignore cursor events: {}", e);
        return;
    }

    #[cfg(debug_assertions)]
    webview_window.open_devtools();

    info!("Scene window initialized successfully.");
}
