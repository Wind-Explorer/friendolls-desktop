use tauri::Emitter;
use tracing::error;

use crate::get_app_handle;
use crate::{lock_r, state::FDOLL};

pub use types::*;
mod icon_cache;
#[cfg(target_os = "macos")]
mod macos;
mod types;
#[cfg(target_os = "windows")]
mod windows;

/// Listens for changes in the active (foreground) application and calls the provided callback with metadata.
/// The implementation varies by platform: macOS uses NSWorkspace notifications, Windows uses WinEventHook.
pub fn listen_for_active_app_changes<F>(callback: F)
where
    F: Fn(AppMetadata) + Send + 'static,
{
    listen_impl(callback)
}

#[cfg(target_os = "macos")]
fn listen_impl<F>(callback: F)
where
    F: Fn(AppMetadata) + Send + 'static,
{
    macos::listen_for_active_app_changes(callback);
}

#[cfg(target_os = "windows")]
fn listen_impl<F>(callback: F)
where
    F: Fn(AppMetadata) + Send + 'static,
{
    windows::listen_for_active_app_changes(callback);
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn listen_impl<F>(_callback: F)
where
    F: Fn(AppMetadata) + Send + 'static,
{
    // no-op on unsupported platforms
}

pub static ACTIVE_APP_CHANGED: &str = "active-app-changed";

/// Initializes the foreground app change listener
/// and emits events to the Tauri app on changes.
/// Used for app to emit user foreground app to peers.
pub fn init_foreground_app_change_listener() {
    let app_handle = get_app_handle();
    listen_for_active_app_changes(|app_metadata: AppMetadata| {
        {
            let guard = lock_r!(FDOLL);
            if guard
                .network
                .clients
                .as_ref()
                .map(|c| c.is_ws_initialized)
                .unwrap_or(false)
            {
                // Check if app metadata has valid data
                let has_valid_name = app_metadata
                    .localized
                    .as_ref()
                    .or(app_metadata.unlocalized.as_ref())
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false);
                
                if has_valid_name {
                    let payload = crate::services::ws::UserStatusPayload {
                        app_metadata: app_metadata.clone(),
                        state: "idle".to_string(),
                    };
                    tauri::async_runtime::spawn(async move {
                        crate::services::ws::report_user_status(payload).await;
                    });
                }
            }
        };
        if let Err(e) = app_handle.emit(ACTIVE_APP_CHANGED, app_metadata) {
            error!("Failed to emit active app changed event: {}", e);
        };
    });
}
