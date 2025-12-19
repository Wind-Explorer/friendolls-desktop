use rust_socketio::{ClientBuilder, Payload, RawClient};
use serde_json::json;
use tauri::{async_runtime, Emitter};
use tracing::{error, info};

use crate::{
    get_app_handle, lock_r, lock_w,
    models::app_config::AppConfig,
    services::cursor::{normalized_to_absolute, CursorPosition, CursorPositions},
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
    pub const FRIEND_DOLL_CREATED: &str = "friend-doll-created";
    pub const FRIEND_DOLL_UPDATED: &str = "friend-doll-updated";
    pub const FRIEND_DOLL_DELETED: &str = "friend-doll-deleted";
}

fn on_friend_request_received(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(str) => {
            println!("Received friend request: {:?}", str);
            if let Err(e) = get_app_handle().emit(WS_EVENT::FRIEND_REQUEST_RECEIVED, str) {
                error!("Failed to emit friend request received event: {:?}", e);
            }
        }
        _ => error!("Received unexpected payload format for friend request received"),
    }
}

fn on_friend_request_accepted(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(str) => {
            println!("Received friend request accepted: {:?}", str);
            if let Err(e) = get_app_handle().emit(WS_EVENT::FRIEND_REQUEST_ACCEPTED, str) {
                error!("Failed to emit friend request accepted event: {:?}", e);
            }
        }
        _ => error!("Received unexpected payload format for friend request accepted"),
    }
}

fn on_friend_request_denied(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(str) => {
            println!("Received friend request denied: {:?}", str);
            if let Err(e) = get_app_handle().emit(WS_EVENT::FRIEND_REQUEST_DENIED, str) {
                error!("Failed to emit friend request denied event: {:?}", e);
            }
        }
        _ => error!("Received unexpected payload format for friend request denied"),
    }
}

fn on_unfriended(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(str) => {
            println!("Received unfriended: {:?}", str);
            if let Err(e) = get_app_handle().emit(WS_EVENT::UNFRIENDED, str) {
                error!("Failed to emit unfriended event: {:?}", e);
            }
        }
        _ => error!("Received unexpected payload format for unfriended"),
    }
}

fn on_friend_cursor_position(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            // values is Vec<serde_json::Value>
            if let Some(first_value) = values.first() {
                let incoming_data: Result<IncomingFriendCursorPayload, _> =
                    serde_json::from_value(first_value.clone());

                match incoming_data {
                    Ok(friend_data) => {
                        // We received normalized coordinates (mapped)
                        let mapped_pos = &friend_data.position;

                        // Convert normalized coordinates back to absolute screen coordinates (raw)
                        let raw_pos = normalized_to_absolute(mapped_pos);

                        let outgoing_payload = OutgoingFriendCursorPayload {
                            user_id: friend_data.user_id.clone(),
                            position: CursorPositions {
                                raw: raw_pos,
                                mapped: mapped_pos.clone(),
                            },
                        };

                        if let Err(e) = get_app_handle()
                            .emit(WS_EVENT::FRIEND_CURSOR_POSITION, outgoing_payload)
                        {
                            error!("Failed to emit friend cursor position event: {:?}", e);
                        }
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
            if let Err(e) = get_app_handle().emit(WS_EVENT::FRIEND_DISCONNECTED, str) {
                error!("Failed to emit friend disconnected event: {:?}", e);
            }
        }
        _ => error!("Received unexpected payload format for friend disconnected"),
    }
}

fn on_friend_doll_created(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            // Log raw JSON for now, as requested
            if let Some(first_value) = values.first() {
                info!("Received friend-doll-created event: {:?}", first_value);
                // Future: Trigger re-fetch or emit to frontend
            } else {
                 info!("Received friend-doll-created event with empty payload");
            }
        }
         _ => error!("Received unexpected payload format for friend-doll-created"),
    }
}

fn on_friend_doll_updated(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                info!("Received friend-doll-updated event: {:?}", first_value);
            } else {
                 info!("Received friend-doll-updated event with empty payload");
            }
        }
        _ => error!("Received unexpected payload format for friend-doll-updated"),
    }
}

fn on_friend_doll_deleted(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                info!("Received friend-doll-deleted event: {:?}", first_value);
            } else {
                 info!("Received friend-doll-deleted event with empty payload");
            }
        }
        _ => error!("Received unexpected payload format for friend-doll-deleted"),
    }
}

pub async fn report_cursor_data(cursor_position: CursorPosition) {
    // Only attempt to get clients if lock_r succeeds (it should, but safety first)
    // and if clients are actually initialized.
    let client_opt = {
        let guard = lock_r!(FDOLL);
        guard
            .clients
            .as_ref()
            .and_then(|c| c.ws_client.as_ref())
            .cloned()
    };

    if let Some(client) = client_opt {
        match async_runtime::spawn_blocking(move || {
            client.emit(
                WS_EVENT::CURSOR_REPORT_POSITION,
                Payload::Text(vec![json!(cursor_position)]),
            )
        })
        .await
        {
            Ok(Ok(_)) => (),
            Ok(Err(e)) => error!("Failed to emit cursor report: {}", e),
            Err(e) => error!("Failed to execute blocking task for cursor report: {}", e),
        }
    } else {
        // Quietly fail if client is not ready (e.g. during startup/shutdown)
        // or debug log it.
        // debug!("WebSocket client not ready to report cursor data");
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
            if let Some(clients) = guard.clients.as_mut() {
                clients.ws_client = Some(ws_client);
            }
            info!("WebSocket client initialized after authentication");
        }
        Err(e) => {
            error!("Failed to initialize WebSocket client: {}", e);
            // We should probably inform the user or retry here, but for now we just log it.
        }
    }
}

pub async fn build_ws_client(
    app_config: &AppConfig,
) -> Result<rust_socketio::client::Client, String> {
    let token_result = crate::services::auth::get_access_token().await;

    let token = match token_result {
        Some(t) => t,
        None => return Err("No access token available for WebSocket connection".to_string()),
    };

    info!("Building WebSocket client with authentication");

    let api_base_url = app_config
        .api_base_url
        .clone()
        .ok_or("Missing API base URL")?;

    let client_result = async_runtime::spawn_blocking(move || {
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
            .on(WS_EVENT::FRIEND_DOLL_CREATED, on_friend_doll_created)
            .on(WS_EVENT::FRIEND_DOLL_UPDATED, on_friend_doll_updated)
            .on(WS_EVENT::FRIEND_DOLL_DELETED, on_friend_doll_deleted)
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
