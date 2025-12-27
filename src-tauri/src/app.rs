use std::time::Duration;

use tokio::time::{sleep, Instant};
use tracing::info;

use crate::{
    services::{
        auth::{get_access_token, get_tokens},
        scene::{close_splash_window, open_scene_window, open_splash_window},
        ws::init_ws_client,
    },
    state::init_app_data,
    system_tray::init_system_tray,
};

pub async fn start_fdoll() {
    init_system_tray();
    bootstrap().await;
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
        Some(tokens) => {
            info!("Tokens found in keyring - restoring user session");
            construct_app().await;
        }
        None => {
            info!("No active session found - user needs to authenticate");
            match crate::services::auth::init_auth_code_retrieval(|| {
                info!("Authentication successful, creating scene...");
                tauri::async_runtime::spawn(async {
                    construct_app().await;
                });
            }) {
                Ok(it) => it,
                Err(err) => todo!("Handle authentication error: {}", err),
            };
        }
    }
}
