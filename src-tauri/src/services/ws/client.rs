use std::time::Duration;

use rust_socketio::ClientBuilder;
use tauri::async_runtime;
use tokio::time::sleep;
use tracing::{error, info};

use crate::{
    lock_r, lock_w,
    services::{auth::get_access_token, client_config::AppConfig},
    state::FDOLL,
};

use super::handlers;

pub async fn establish_websocket_connection() {
    const MAX_ATTEMPTS: u8 = 5;
    const BACKOFF: Duration = Duration::from_millis(300);

    for _attempt in 1..=MAX_ATTEMPTS {
        if get_access_token().await.is_some() {
            if init_ws_client().await {
                return; // Success
            } else {
                // Connection failed, trigger disaster recovery
                crate::services::session::handle_disastrous_failure(Some(
                    "WebSocket connection failed. Please check your network and try again."
                        .to_string(),
                ))
                .await;
                return;
            }
        }

        sleep(BACKOFF).await;
    }

    // If we exhausted retries without valid token
    crate::services::session::handle_disastrous_failure(Some(
        "Failed to authenticate. Please restart and sign in again.".to_string(),
    ))
    .await;
}

pub async fn init_ws_client() -> bool {
    let app_config = {
        let guard = lock_r!(FDOLL);
        guard.app_config.clone()
    };

    match build_ws_client(&app_config).await {
        Ok(ws_client) => {
            let mut guard = lock_w!(FDOLL);
            if let Some(clients) = guard.network.clients.as_mut() {
                clients.ws_client = Some(ws_client);
                clients.is_ws_initialized = false; // wait for initialized event
            }
            true
        }
        Err(e) => {
            error!("Failed to initialize WebSocket client: {}", e);
            clear_ws_client().await;
            false
        }
    }
}

pub async fn clear_ws_client() {
    let mut guard = lock_w!(FDOLL);
    if let Some(clients) = guard.network.clients.as_mut() {
        clients.ws_client = None;
        clients.is_ws_initialized = false;
        clients.ws_emit_failures = 0;
    }
}

pub async fn build_ws_client(
    app_config: &AppConfig,
) -> Result<rust_socketio::client::Client, String> {
    // Always fetch a fresh/valid token (refreshing if needed)
    let token = match crate::services::auth::get_access_token().await {
        Some(t) => t,
        None => return Err("No access token available for WebSocket connection".to_string()),
    };

    let api_base_url = app_config
        .api_base_url
        .clone()
        .ok_or("Missing API base URL")?;

    let client_result = async_runtime::spawn_blocking(move || {
        let builder = ClientBuilder::new(api_base_url)
            .namespace("/")
            .auth(serde_json::json!({ "token": token }));

        let builder_with_handlers = handlers::register_event_handlers(builder);

        builder_with_handlers.connect()
    })
    .await;

    match client_result {
        Ok(connect_result) => match connect_result {
            Ok(c) => {
                info!("WebSocket client connected successfully");
                Ok(c)
            }
            Err(e) => {
                let err_msg = format!("Failed to connect WebSocket: {:?}", e);
                error!("{}", err_msg);
                Err(err_msg)
            }
        },
        Err(e) => {
            let err_msg = format!("Failed to spawn blocking task: {:?}", e);
            error!("{}", err_msg);
            Err(err_msg)
        }
    }
}
