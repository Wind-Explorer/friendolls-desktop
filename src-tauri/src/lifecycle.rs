use crate::{
    lock_w,
    services::{
        active_app::init_active_app_changes_listener,
        health_manager::show_health_manager_with_error,
    },
    startup::init_startup_sequence,
    state::FDOLL,
    system_tray::init_system_tray,
};

/// Initializes and starts the core app lifecycle after initial setup.
///
/// This function handles:
/// - System tray initialization and storage in app state
/// - Active app change listener setup
/// - Startup sequence execution with error handling
///
/// # Errors
/// If the startup sequence fails, displays a health manager dialog
/// with the error details.
///
/// # Example
/// ```
/// // Called automatically during app setup in setup_fdoll()
/// lifecycle::start_fdoll().await;
/// ```
pub async fn start_fdoll() {
    let tray = init_system_tray();
    {
        let mut guard = lock_w!(FDOLL);
        guard.tray = Some(tray);
    }

    // Begin listening for foreground app changes
    init_active_app_changes_listener();

    if let Err(err) = init_startup_sequence().await {
        tracing::warn!("Startup sequence encountered an error: {}", err);
        show_health_manager_with_error(Some(err.to_string()));
    }
}
