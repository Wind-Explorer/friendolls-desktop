use crate::get_app_handle;

pub static SCENE_WINDOW_LABEL: &str = "scene";

pub fn overlay_fullscreen(window: &tauri::Window) -> Result<(), tauri::Error> {
    // Get the primary monitor
    let monitor = get_app_handle().primary_monitor()?.unwrap();
    let monitor_position = monitor.position();
    let monitor_size = monitor.size();

    // Set window position to top-left
    window.set_position(tauri::PhysicalPosition {
        x: monitor_position.x,
        y: monitor_position.y,
    })?;

    // Set window size to match screen size
    window.set_size(tauri::PhysicalSize {
        width: monitor_size.width,
        height: monitor_size.height,
    })?;

    Ok(())
}
