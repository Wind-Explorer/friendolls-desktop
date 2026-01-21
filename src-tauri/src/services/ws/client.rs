use rust_socketio::{ClientBuilder, Event, Payload, RawClient};
use serde_json::json;
use tauri::async_runtime;
use tracing::{error, info};

use crate::{
    lock_r, lock_w,
    services::{
        client_config_manager::AppConfig, health_manager::close_health_manager_window,
        scene::open_scene_window,
    },
    state::FDOLL,
};

use super::WS_EVENT;

fn emit_initialize(socket: &RawClient) {
    if let Err(e) = socket.emit(WS_EVENT::CLIENT_INITIALIZE, json!({})) {
        error!("Failed to emit client-initialize: {:?}", e);
    }
}

fn on_connected(_payload: Payload, socket: RawClient) {
    info!("WebSocket connected. Sending initialization request.");
    emit_initialize(&socket);
}

fn on_initialized(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                info!("Received initialized event: {:?}", first_value);

                // Mark WebSocket as initialized and reset backoff timer
                let mut guard = lock_w!(FDOLL);
                if let Some(clients) = guard.network.clients.as_mut() {
                    clients.is_ws_initialized = true;
                }

                // Connection restored: close health manager and reopen scene
                close_health_manager_window();
                open_scene_window();
            } else {
                info!("Received initialized event with empty payload");
            }
        }
        _ => error!("Received unexpected payload format for initialized"),
    }
}

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
        ClientBuilder::new(api_base_url)
            .namespace("/")
            .on(
                WS_EVENT::FRIEND_REQUEST_RECEIVED,
                super::friend::on_friend_request_received,
            )
            .on(
                WS_EVENT::FRIEND_REQUEST_ACCEPTED,
                super::friend::on_friend_request_accepted,
            )
            .on(
                WS_EVENT::FRIEND_REQUEST_DENIED,
                super::friend::on_friend_request_denied,
            )
            .on(WS_EVENT::UNFRIENDED, super::friend::on_unfriended)
            .on(
                WS_EVENT::FRIEND_CURSOR_POSITION,
                super::friend::on_friend_cursor_position,
            )
            .on(
                WS_EVENT::FRIEND_DISCONNECTED,
                super::friend::on_friend_disconnected,
            )
            .on(
                WS_EVENT::FRIEND_DOLL_CREATED,
                super::friend::on_friend_doll_created,
            )
            .on(
                WS_EVENT::FRIEND_DOLL_UPDATED,
                super::friend::on_friend_doll_updated,
            )
            .on(
                WS_EVENT::FRIEND_DOLL_DELETED,
                super::friend::on_friend_doll_deleted,
            )
            .on(
                WS_EVENT::FRIEND_ACTIVE_DOLL_CHANGED,
                super::friend::on_friend_active_doll_changed,
            )
            .on(WS_EVENT::DOLL_CREATED, super::doll::on_doll_created)
            .on(WS_EVENT::DOLL_UPDATED, super::doll::on_doll_updated)
            .on(WS_EVENT::DOLL_DELETED, super::doll::on_doll_deleted)
            .on(WS_EVENT::INITIALIZED, on_initialized)
            .on(
                WS_EVENT::INTERACTION_RECEIVED,
                super::interaction::on_interaction_received,
            )
            .on(
                WS_EVENT::INTERACTION_DELIVERY_FAILED,
                super::interaction::on_interaction_delivery_failed,
            )
            // rust-socketio fires Event::Connect on initial connect AND reconnects
            // so we resend initialization there instead of a dedicated reconnect event.
            .on(Event::Connect, on_connected)
            .auth(json!({ "token": token }))
            .connect()
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
