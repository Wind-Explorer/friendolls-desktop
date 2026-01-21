use reqwest::StatusCode;
use std::time::Duration;
use tokio::time::{sleep, Instant};
use tracing::{info, warn};

use crate::{
    lock_w,
    models::health::HealthError,
    remotes::health::HealthRemote,
    services::{
        active_app::init_active_app_changes_listener,
        auth::{get_access_token, get_tokens},
        health_manager::show_health_manager_with_error,
        scene::{close_splash_window, open_scene_window, open_splash_window},
        welcome::open_welcome_window,
        ws::init_ws_client,
    },
    state::{init_app_data, FDOLL},
    system_tray::{init_system_tray, update_system_tray},
};

pub async fn start_fdoll() {
    let tray = init_system_tray();
    {
        let mut guard = lock_w!(FDOLL);
        guard.tray = Some(tray);
    }

    // Begin listening for foreground app changes
    init_active_app_changes_listener();

    if let Err(err) = init_startup_sequence().await {
        tracing::error!("startup sequence failed: {err}");
        show_health_manager_with_error(Some(err.to_string()));
    }
}

async fn init_ws_after_auth() {
    const MAX_ATTEMPTS: u8 = 5;
    const BACKOFF: Duration = Duration::from_millis(300);

    for _attempt in 1..=MAX_ATTEMPTS {
        if get_access_token().await.is_some() {
            init_ws_client().await;
            return;
        }

        sleep(BACKOFF).await;
    }
}

async fn construct_app() {
    open_splash_window();

    // Record start time for minimum splash duration
    let start = Instant::now();

    // Initialize app data first so we only start WebSocket after auth is fully available
    init_app_data().await;

    // Initialize WebSocket client after we know auth is present
    init_ws_after_auth().await;

    // Ensure splash stays visible for at least 3 seconds
    let elapsed = start.elapsed();
    if elapsed < Duration::from_secs(3) {
        sleep(Duration::from_secs(3) - elapsed).await;
    }

    // Close splash and open main scene
    close_splash_window();
    open_scene_window();
}

pub async fn bootstrap() {
    match get_tokens().await {
        Some(_tokens) => {
            info!("Tokens found in keyring - restoring user session");
            construct_app().await;
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

/// Perform checks for environment, network condition
/// and handle situations where startup would not be appropriate.
async fn init_startup_sequence() -> Result<(), HealthError> {
    let health_remote = HealthRemote::try_new()?;

    // simple retry loop to smooth transient network issues
    const MAX_ATTEMPTS: u8 = 3;
    const BACKOFF_MS: u64 = 500;

    for attempt in 1..=MAX_ATTEMPTS {
        match health_remote.get_health().await {
            Ok(_) => {
                bootstrap().await;
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
