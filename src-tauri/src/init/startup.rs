use std::time::Duration;
use tokio::time::{sleep, Instant};

use crate::{
    services::{
        auth::get_access_token,
        scene::{close_splash_window, open_scene_window, open_splash_window},
        ws::init_ws_client,
    },
    state::init_app_data,
};

async fn establish_websocket_connection() {
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

pub async fn initialize_app_data_and_connections() -> Instant {
    open_splash_window();

    // Record start time for minimum splash duration
    let start = Instant::now();

    // Initialize app data first so we only start WebSocket after auth is fully available
    init_app_data().await;

    // Initialize WebSocket client after we know auth is present
    establish_websocket_connection().await;

    start
}

pub async fn transition_to_main_interface(start: Instant) {
    // Ensure splash stays visible for at least 3 seconds
    let elapsed = start.elapsed();
    if elapsed < Duration::from_secs(3) {
        sleep(Duration::from_secs(3) - elapsed).await;
    }

    // Close splash and open main scene
    close_splash_window();
    open_scene_window();
}