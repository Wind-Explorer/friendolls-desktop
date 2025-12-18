use std::time::Duration;
use tokio::time::{sleep, Instant};
use tracing::info;

use crate::{
    services::{
        auth::get_tokens,
        scene::{close_splash_window, open_scene_window, open_splash_window},
    },
    state::init_app_data,
    system_tray::init_system_tray,
};

pub async fn start_fdoll() {
    init_system_tray();
    bootstrap().await;
}

async fn construct_app() {
    open_splash_window();

    // Record start time for minimum splash duration
    let start = Instant::now();

    // Spawn initialization tasks in parallel
    // We want to wait for them to finish, but they run concurrently
    let init_data = tauri::async_runtime::spawn(async {
        init_app_data().await;
    });

    let init_ws = tauri::async_runtime::spawn(async {
        crate::services::ws::init_ws_client().await;
    });

    // Wait for both to complete
    let _ = tokio::join!(init_data, init_ws);

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
        Some(_) => {
            info!("User session restored");
            construct_app().await;
        }
        None => {
            info!("No active session, user needs to authenticate");
            match crate::services::auth::init_auth_code_retrieval(|| {
                info!("Authentication successful, creating scene...");
                tauri::async_runtime::spawn(async {
                    info!("Creating scene after auth success...");
                    construct_app().await;
                });
            }) {
                Ok(it) => it,
                Err(err) => todo!("Handle authentication error: {}", err),
            };
        }
    }
}
