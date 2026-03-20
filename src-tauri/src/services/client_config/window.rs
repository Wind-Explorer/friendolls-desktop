use tracing::error;

use crate::services::window_manager::{
    ensure_window, EnsureWindowError, EnsureWindowResult, WindowConfig,
};

use super::{ClientConfigError, CLIENT_CONFIG_WINDOW_LABEL};

#[tauri::command]
pub fn open_config_window() -> Result<(), ClientConfigError> {
    let mut config = WindowConfig::regular_ui(
        CLIENT_CONFIG_WINDOW_LABEL,
        "/client-config",
        "Advanced Configuration",
    );
    config.width = 300.0;
    config.height = 420.0;
    config.visible = false;

    match ensure_window(&config, true, true) {
        Ok(EnsureWindowResult::Created(window)) => {
            if let Err(e) = window.show() {
                error!("Failed to show client config window: {}", e);
                return Err(ClientConfigError::ShowWindow(e));
            }
            if let Err(e) = window.set_focus() {
                error!("Failed to focus client config window: {e}");
            }
            Ok(())
        }
        Ok(EnsureWindowResult::Existing(_)) => Ok(()),
        Err(EnsureWindowError::MissingParent(parent_label)) => {
            error!(
                "Missing parent '{}' for client config window: impossible state",
                parent_label
            );
            Err(ClientConfigError::MissingParent(parent_label))
        }
        Err(EnsureWindowError::ShowExisting(e)) => {
            error!("Failed to show client config window: {e}");
            Err(ClientConfigError::ShowWindow(e))
        }
        Err(EnsureWindowError::SetParent(e)) => {
            error!("Failed to set parent for client config window: {}", e);
            Err(ClientConfigError::Window(e))
        }
        Err(EnsureWindowError::Build(e)) => {
            error!("Failed to build client config window: {}", e);
            Err(ClientConfigError::Window(e))
        }
    }
}
