use reqwest::StatusCode;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::{
    init::startup::{initialize_app_data_and_connections, transition_to_main_interface},
    lock_w,
    models::health::HealthError,
    remotes::health::HealthRemote,
    services::{
        active_app::init_active_app_changes_listener,
        auth::get_tokens,
        health_manager::show_health_manager_with_error,
        scene::close_splash_window,
        welcome::open_welcome_window,
    },
    state::FDOLL,
    system_tray::{init_system_tray, update_system_tray},
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
/// // Called automatically during app setup in initialize_app_environment()
/// lifecycle::launch_core_services().await;
/// ```
pub async fn launch_core_services() {
    let tray = init_system_tray();
    {
        let mut guard = lock_w!(FDOLL);
        guard.tray = Some(tray);
    }

    // Begin listening for foreground app changes
    init_active_app_changes_listener();

    if let Err(err) = validate_environment_and_start_app().await {
        tracing::warn!("Startup sequence encountered an error: {}", err);
        show_health_manager_with_error(Some(err.to_string()));
    }
}

/// Perform checks for environment, network condition
/// and handle situations where startup would not be appropriate.
pub async fn validate_environment_and_start_app() -> Result<(), HealthError> {
    let health_remote = HealthRemote::try_new()?;

    // simple retry loop to smooth transient network issues
    const MAX_ATTEMPTS: u8 = 3;
    const BACKOFF_MS: u64 = 500;

    for attempt in 1..=MAX_ATTEMPTS {
        match health_remote.get_health().await {
            Ok(_) => {
                handle_authentication_flow().await;
                return Ok(());
            }
            Err(HealthError::NonOkStatus(status)) => {
                warn!(attempt, "server health reported non-OK status: {status}");
                return Err(HealthError::NonOkStatus(status));
            }
            Err(HealthError::UnexpectedStatus(status)) => {
                warn!(attempt, "server health check failed with status: {status}");
                return Err(HealthError::UnexpectedStatus(status));
            }
            Err(err) => {
                warn!(attempt, "server health check failed: {err}");
                if attempt == MAX_ATTEMPTS {
                    return Err(err);
                }
            }
        }

        if attempt < MAX_ATTEMPTS {
            sleep(Duration::from_millis(BACKOFF_MS)).await;
        }
    }

    Err(HealthError::UnexpectedStatus(
        StatusCode::SERVICE_UNAVAILABLE,
    ))
}

/// Handles authentication flow: checks for tokens and either restores session or shows welcome.
pub async fn handle_authentication_flow() {
    match get_tokens().await {
        Some(_tokens) => {
            info!("Tokens found in keyring - restoring user session");
            let start = initialize_app_data_and_connections().await;
            transition_to_main_interface(start).await;
            update_system_tray(true);
        }
        None => {
            info!("No active session found - showing welcome first");
            open_welcome_window();
            close_splash_window();
            update_system_tray(false);
        }
    }
}
