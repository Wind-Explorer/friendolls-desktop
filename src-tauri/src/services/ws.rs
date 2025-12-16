use rust_socketio::{ClientBuilder, Payload, RawClient};
use serde_json::json;
use tauri::{async_runtime, Emitter};
use tracing::{error, info};

use crate::{
    get_app_handle, lock_r, lock_w, models::app_config::AppConfig,
    services::cursor::{grid_to_absolute_position, CursorPosition, CursorPositions},
    state::FDOLL,
};
use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)] // pretend to be a const like in js
pub struct WS_EVENT;

#[derive(Debug, Deserialize)]
struct IncomingFriendCursorPayload {
    #[serde(rename = "userId")]
    user_id: String,
    position: CursorPosition,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OutgoingFriendCursorPayload {
    user_id: String,
    position: CursorPositions,
}

impl WS_EVENT {
    pub const CURSOR_REPORT_POSITION: &str = "cursor-report-position";
    pub const FRIEND_REQUEST_RECEIVED: &str = "friend-request-received";
    pub const FRIEND_REQUEST_ACCEPTED: &str = "friend-request-accepted";
    pub const FRIEND_REQUEST_DENIED: &str = "friend-request-denied";
    pub const UNFRIENDED: &str = "unfriended";
    pub const FRIEND_CURSOR_POSITION: &str = "friend-cursor-position";
    pub const FRIEND_DISCONNECTED: &str = "friend-disconnected";
}

fn on_friend_request_received(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(str) => {
            println!("Received friend request: {:?}", str);
            get_app_handle()
                .emit(WS_EVENT::FRIEND_REQUEST_RECEIVED, str)
                .unwrap();
        }
        _ => error!("Received unexpected payload format for friend request received"),
    }
}

fn on_friend_request_accepted(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(str) => {
            println!("Received friend request accepted: {:?}", str);
            get_app_handle()
                .emit(WS_EVENT::FRIEND_REQUEST_ACCEPTED, str)
                .unwrap();
        }
        _ => error!("Received unexpected payload format for friend request accepted"),
    }
}

fn on_friend_request_denied(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(str) => {
            println!("Received friend request denied: {:?}", str);
            get_app_handle()
                .emit(WS_EVENT::FRIEND_REQUEST_DENIED, str)
                .unwrap();
        }
        _ => error!("Received unexpected payload format for friend request denied"),
    }
}

fn on_unfriended(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(str) => {
            println!("Received unfriended: {:?}", str);
            get_app_handle().emit(WS_EVENT::UNFRIENDED, str).unwrap();
        }
        _ => error!("Received unexpected payload format for unfriended"),
    }
}

fn on_friend_cursor_position(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            // values is Vec<serde_json::Value>
            if let Some(first_value) = values.first() {
                 let incoming_data: Result<IncomingFriendCursorPayload, _> = serde_json::from_value(first_value.clone());

                 match incoming_data {
                    Ok(friend_data) => {
                         // We received grid coordinates (mapped)
                        let mapped_pos = &friend_data.position;

                        // Convert grid coordinates back to absolute screen coordinates (raw)
                        let raw_pos = grid_to_absolute_position(mapped_pos);

                        let outgoing_payload = OutgoingFriendCursorPayload {
                            user_id: friend_data.user_id.clone(),
                            position: CursorPositions {
                                raw: raw_pos,
                                mapped: mapped_pos.clone(),
                            },
                        };

                        get_app_handle()
                            .emit(WS_EVENT::FRIEND_CURSOR_POSITION, outgoing_payload)
                            .unwrap();
                    }
                    Err(e) => {
                        error!("Failed to parse friend cursor position data: {}", e);
                    }
                 }
            } else {
                error!("Received empty text payload for friend cursor position");
            }
        }
        _ => error!("Received unexpected payload format for friend cursor position"),
    }
}

fn on_friend_disconnected(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(str) => {
            println!("Received friend disconnected: {:?}", str);
            get_app_handle()
                .emit(WS_EVENT::FRIEND_DISCONNECTED, str)
                .unwrap();
        }
        _ => error!("Received unexpected payload format for friend disconnected"),
    }
}

pub async fn report_cursor_data(cursor_position: CursorPosition) {
    let client = {
        let guard = lock_r!(FDOLL);
        guard
            .clients
            .as_ref()
            .expect("Clients are initialized")
            .ws_client
            .as_ref()
            .expect("WebSocket client is initialized")
            .clone()
    };

    match async_runtime::spawn_blocking(move || {
        client.emit(
            WS_EVENT::CURSOR_REPORT_POSITION,
            Payload::Text(vec![json!(cursor_position)]),
        )
    })
    .await
    {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => error!("Failed to emit ping: {}", e),
        Err(e) => error!("Failed to execute blocking task: {}", e),
    }
}

pub async fn init_ws_client() {
    let app_config = {
        let guard = lock_r!(FDOLL);
        guard.app_config.clone()
    };

    let ws_client = build_ws_client(&app_config).await;

    let mut guard = lock_w!(FDOLL);
    if let Some(clients) = guard.clients.as_mut() {
        clients.ws_client = Some(ws_client);
    }
    info!("WebSocket client initialized after authentication");
}

pub async fn build_ws_client(app_config: &AppConfig) -> rust_socketio::client::Client {
    let token = crate::services::auth::get_access_token()
        .await
        .expect("No access token available for WebSocket connection");

    info!("Building WebSocket client with authentication");

    let api_base_url = app_config
        .api_base_url
        .clone()
        .expect("Missing API base URL");

    let client = async_runtime::spawn_blocking(move || {
        ClientBuilder::new(api_base_url)
            .namespace("/")
            .on(
                WS_EVENT::FRIEND_REQUEST_RECEIVED,
                on_friend_request_received,
            )
            .on(
                WS_EVENT::FRIEND_REQUEST_ACCEPTED,
                on_friend_request_accepted,
            )
            .on(WS_EVENT::FRIEND_REQUEST_DENIED, on_friend_request_denied)
            .on(WS_EVENT::UNFRIENDED, on_unfriended)
            .on(WS_EVENT::FRIEND_CURSOR_POSITION, on_friend_cursor_position)
            .on(WS_EVENT::FRIEND_DISCONNECTED, on_friend_disconnected)
            .auth(json!({ "token": token }))
            .connect()
    })
    .await
    .expect("Failed to spawn blocking task");

    match client {
        Ok(c) => {
            info!("WebSocket client connected successfully");
            c
        }
        Err(e) => {
            error!("Failed to connect WebSocket: {:?}", e);
            panic!(
                "TODO: better error handling for WebSocket connection - {}",
                e
            );
        }
    }
}
