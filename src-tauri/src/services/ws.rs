use rust_socketio::{ClientBuilder, Event, Payload, RawClient};
use serde_json::json;
use tauri::{async_runtime, Emitter};
use tracing::{error, info};

use crate::{
    get_app_handle, lock_r, lock_w,
    models::interaction::{InteractionDeliveryFailedDto, InteractionPayloadDto},
    services::{
        client_config_manager::AppConfig,
        cursor::{normalized_to_absolute, CursorPosition, CursorPositions},
        health_manager::{close_health_manager_window, show_health_manager_with_error},
        scene::open_scene_window,
    },
    state::{init_app_data_scoped, AppDataRefreshScope, FDOLL},
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
    pub const FRIEND_ACTIVE_DOLL_CHANGED: &str = "friend-active-doll-changed";
    pub const DOLL_CREATED: &str = "doll_created";
    pub const DOLL_UPDATED: &str = "doll_updated";
    pub const DOLL_DELETED: &str = "doll_deleted";
    pub const CLIENT_INITIALIZE: &str = "client-initialize";
    pub const INITIALIZED: &str = "initialized";
    pub const INTERACTION_RECEIVED: &str = "interaction-received";
    pub const INTERACTION_DELIVERY_FAILED: &str = "interaction-delivery-failed";
    pub const CLIENT_SEND_INTERACTION: &str = "client-send-interaction";
}

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
                if let Some(clients) = guard.clients.as_mut() {
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

            // Refresh friends list only (optimized - no need to fetch user profile)
            tauri::async_runtime::spawn(async {
                init_app_data_scoped(AppDataRefreshScope::Friends).await;
            });
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

            // Refresh friends list only (optimized - no need to fetch user profile)
            tauri::async_runtime::spawn(async {
                init_app_data_scoped(AppDataRefreshScope::Friends).await;
            });
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

fn on_friend_active_doll_changed(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                info!(
                    "Received friend-active-doll-changed event: {:?}",
                    first_value
                );
                if let Err(e) =
                    get_app_handle().emit(WS_EVENT::FRIEND_ACTIVE_DOLL_CHANGED, first_value)
                {
                    error!("Failed to emit friend-active-doll-changed event: {:?}", e);
                }

                // Refresh friends list only (optimized - friend's active doll is part of friends data)
                // Deduplicate burst events inside init_app_data_scoped.
                tauri::async_runtime::spawn(async {
                    init_app_data_scoped(AppDataRefreshScope::Friends).await;
                });
            } else {
                info!("Received friend-active-doll-changed event with empty payload");
            }
        }
        _ => error!("Received unexpected payload format for friend-active-doll-changed"),
    }
}

fn on_doll_created(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                info!("Received doll.created event: {:?}", first_value);

                // Refresh dolls list
                tauri::async_runtime::spawn(async {
                    init_app_data_scoped(AppDataRefreshScope::Dolls).await;
                });
            } else {
                info!("Received doll.created event with empty payload");
            }
        }
        _ => error!("Received unexpected payload format for doll.created"),
    }
}

fn on_doll_updated(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                info!("Received doll.updated event: {:?}", first_value);

                // Try to extract doll ID to check if it's the active doll
                let doll_id = first_value.get("id").and_then(|v| v.as_str());

                let is_active_doll = if let Some(id) = doll_id {
                    let guard = lock_r!(FDOLL);
                    guard
                        .app_data
                        .user
                        .as_ref()
                        .and_then(|u| u.active_doll_id.as_ref())
                        .map(|active_id| active_id == id)
                        .unwrap_or(false)
                } else {
                    false
                };

                // Refresh dolls + potentially User/Friends if active doll
                tauri::async_runtime::spawn(async move {
                    init_app_data_scoped(AppDataRefreshScope::Dolls).await;
                    if is_active_doll {
                        init_app_data_scoped(AppDataRefreshScope::User).await;
                        init_app_data_scoped(AppDataRefreshScope::Friends).await;
                    }
                });
            } else {
                info!("Received doll.updated event with empty payload");
            }
        }
        _ => error!("Received unexpected payload format for doll.updated"),
    }
}

fn on_doll_deleted(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                info!("Received doll.deleted event: {:?}", first_value);

                // Try to extract doll ID to check if it was the active doll
                let doll_id = first_value.get("id").and_then(|v| v.as_str());

                let is_active_doll = if let Some(id) = doll_id {
                    let guard = lock_r!(FDOLL);
                    guard
                        .app_data
                        .user
                        .as_ref()
                        .and_then(|u| u.active_doll_id.as_ref())
                        .map(|active_id| active_id == id)
                        .unwrap_or(false)
                } else {
                    false
                };

                // Refresh dolls + User/Friends if the deleted doll was active
                tauri::async_runtime::spawn(async move {
                    init_app_data_scoped(AppDataRefreshScope::Dolls).await;
                    if is_active_doll {
                        init_app_data_scoped(AppDataRefreshScope::User).await;
                        init_app_data_scoped(AppDataRefreshScope::Friends).await;
                    }
                });
            } else {
                info!("Received doll.deleted event with empty payload");
            }
        }
        _ => error!("Received unexpected payload format for doll.deleted"),
    }
}

fn on_interaction_received(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                info!("Received interaction-received event: {:?}", first_value);

                let interaction_data: Result<InteractionPayloadDto, _> =
                    serde_json::from_value(first_value.clone());

                match interaction_data {
                    Ok(data) => {
                        if let Err(e) = get_app_handle().emit(WS_EVENT::INTERACTION_RECEIVED, data)
                        {
                            error!("Failed to emit interaction-received event: {:?}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse interaction payload: {}", e);
                    }
                }
            } else {
                info!("Received interaction-received event with empty payload");
            }
        }
        _ => error!("Received unexpected payload format for interaction-received"),
    }
}

fn on_interaction_delivery_failed(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                info!(
                    "Received interaction-delivery-failed event: {:?}",
                    first_value
                );

                let failure_data: Result<InteractionDeliveryFailedDto, _> =
                    serde_json::from_value(first_value.clone());

                match failure_data {
                    Ok(data) => {
                        if let Err(e) =
                            get_app_handle().emit(WS_EVENT::INTERACTION_DELIVERY_FAILED, data)
                        {
                            error!("Failed to emit interaction-delivery-failed event: {:?}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse interaction failure payload: {}", e);
                    }
                }
            } else {
                info!("Received interaction-delivery-failed event with empty payload");
            }
        }
        _ => error!("Received unexpected payload format for interaction-delivery-failed"),
    }
}

pub async fn report_cursor_data(cursor_position: CursorPosition) {
    // Only attempt to get clients if lock_r succeeds (it should, but safety first)
    // and if clients are actually initialized.
    let (client_opt, is_initialized) = {
        let guard = lock_r!(FDOLL);
        if let Some(clients) = &guard.clients {
            (
                clients.ws_client.as_ref().cloned(),
                clients.is_ws_initialized,
            )
        } else {
            (None, false)
        }
    };

    if let Some(client) = client_opt {
        if !is_initialized {
            return;
        }

        match async_runtime::spawn_blocking(move || {
            client.emit(
                WS_EVENT::CURSOR_REPORT_POSITION,
                Payload::Text(vec![json!(cursor_position)]),
            )
        })
        .await
        {
            Ok(Ok(_)) => (),
            Ok(Err(e)) => {
                error!("Failed to emit cursor report: {}", e);
                show_health_manager_with_error(Some(format!("WebSocket emit failed: {}", e)));
            }
            Err(e) => {
                error!("Failed to execute blocking task for cursor report: {}", e);
                show_health_manager_with_error(Some(format!("WebSocket task failed: {}", e)));
            }
        }
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
                clients.is_ws_initialized = false; // wait for initialized event
            }
        }
        Err(e) => {
            error!("Failed to initialize WebSocket client: {}", e);
            // If we failed because no token, clear the WS client to avoid stale retries
            let mut guard = lock_w!(FDOLL);
            if let Some(clients) = guard.clients.as_mut() {
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
            .on(
                WS_EVENT::FRIEND_ACTIVE_DOLL_CHANGED,
                on_friend_active_doll_changed,
            )
            .on(WS_EVENT::DOLL_CREATED, on_doll_created)
            .on(WS_EVENT::DOLL_UPDATED, on_doll_updated)
            .on(WS_EVENT::DOLL_DELETED, on_doll_deleted)
            .on(WS_EVENT::INITIALIZED, on_initialized)
            .on(WS_EVENT::INTERACTION_RECEIVED, on_interaction_received)
            .on(
                WS_EVENT::INTERACTION_DELIVERY_FAILED,
                on_interaction_delivery_failed,
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
