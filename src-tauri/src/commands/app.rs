use crate::get_app_handle;

#[tauri::command]
pub fn quit_app() -> Result<(), String> {
    let app_handle = get_app_handle();
    app_handle.exit(0);
    Ok(())
}

#[tauri::command]
pub fn restart_app() {
    let app_handle = get_app_handle();
    app_handle.restart();
}
