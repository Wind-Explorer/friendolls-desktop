use tracing::error;

use crate::services::window_manager::{
    ensure_window, EnsureWindowError, EnsureWindowResult, WindowConfig,
};

pub static APP_MENU_WINDOW_LABEL: &str = "app_menu";

pub fn open_app_menu_window() {
    let mut config = WindowConfig::regular_ui(APP_MENU_WINDOW_LABEL, "/app-menu", "Friendolls");
    config.width = 400.0;
    config.height = 550.0;
    config.resizable = true;

    match ensure_window(&config, true, false) {
        Ok(EnsureWindowResult::Created(_)) => {}
        Ok(EnsureWindowResult::Existing(_)) => {}
        Err(EnsureWindowError::MissingParent(parent_label)) => {
            error!(
                "Failed to build {} window due to missing parent '{}': impossible state",
                APP_MENU_WINDOW_LABEL, parent_label
            );
        }
        Err(EnsureWindowError::ShowExisting(e))
        | Err(EnsureWindowError::SetParent(e))
        | Err(EnsureWindowError::Build(e)) => {
            error!("Failed to build {} window: {}", APP_MENU_WINDOW_LABEL, e);
        }
    }
}
