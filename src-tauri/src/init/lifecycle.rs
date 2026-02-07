use reqwest::StatusCode;
use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

use crate::{
    models::health::HealthError,
    remotes::health::HealthRemote,
    services::{
        close_all_windows,
        health_manager::open_health_manager_window,
        scene::open_scene_window,
        ws::client::{clear_ws_client, establish_websocket_connection},
    },
    state::{
        auth::{start_background_token_refresh, stop_background_token_refresh},
        clear_app_data, init_app_data_scoped, AppDataRefreshScope,
    },
    system_tray::update_system_tray,
};

/// Connects the user profile and opens the scene window.
pub async fn construct_user_session() {
    connect_user_profile().await;
    close_all_windows();
    open_scene_window();
    update_system_tray(true);
}

/// Disconnects the user profile and closes the scene window.
pub async fn destruct_user_session() {
    disconnect_user_profile().await;
    close_all_windows();
    update_system_tray(false);
}

/// Initializes the user profile and establishes a WebSocket connection.
async fn connect_user_profile() {
    init_app_data_scoped(AppDataRefreshScope::All).await;
    establish_websocket_connection().await;
    start_background_token_refresh().await;
}

/// Clears the user profile and WebSocket connection.
async fn disconnect_user_profile() {
    clear_app_data();
    clear_ws_client().await;
    stop_background_token_refresh();
}

/// Destructs the user session and show health manager window
/// with error message, offering troubleshooting options.
pub async fn handle_disasterous_failure(error_message: Option<String>) {
    destruct_user_session().await;
    open_health_manager_window(error_message);
}

/// Pings the server's health endpoint a maximum of
/// three times with a backoff of 500ms between
/// attempts. Return health error if no success.
pub async fn validate_server_health() -> Result<(), HealthError> {
    let health_remote = HealthRemote::try_new()?;

    // simple retry loop to smooth transient network issues
    const MAX_ATTEMPTS: u8 = 3;
    const BACKOFF_MS: u64 = 500;

    for attempt in 1..=MAX_ATTEMPTS {
        match health_remote.get_health().await {
            Ok(_) => {
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

    warn!("Server is unavailable!");

    Err(HealthError::UnexpectedStatus(
        StatusCode::SERVICE_UNAVAILABLE,
    ))
}
