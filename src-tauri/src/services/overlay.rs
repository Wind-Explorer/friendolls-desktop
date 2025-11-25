use crate::get_app_handle;

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
