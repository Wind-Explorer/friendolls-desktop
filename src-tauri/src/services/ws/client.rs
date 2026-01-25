use rust_socketio::ClientBuilder;
use tauri::async_runtime;
use tracing::{error, info};

use crate::{
    lock_r, lock_w,
    services::client_config_manager::AppConfig,
    state::FDOLL,
};

use super::handlers;

pub async fn init_ws_client() {
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
        }
        Err(e) => {
            error!("Failed to initialize WebSocket client: {}", e);
            // If we failed because no token, clear the WS client to avoid stale retries
            let mut guard = lock_w!(FDOLL);
            if let Some(clients) = guard.network.clients.as_mut() {
                clients.ws_client = None;
                clients.is_ws_initialized = false;
            }
        }
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
