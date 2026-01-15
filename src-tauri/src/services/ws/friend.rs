use rust_socketio::{Payload, RawClient};
use tauri::Emitter;
use tracing::{error, info};

use crate::{
    get_app_handle,
    services::cursor::{normalized_to_absolute, CursorPositions},
    state::{init_app_data_scoped, AppDataRefreshScope},
};

use super::{IncomingFriendCursorPayload, OutgoingFriendCursorPayload, WS_EVENT};

pub fn on_friend_request_received(payload: Payload, _socket: RawClient) {
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

pub fn on_friend_request_accepted(payload: Payload, _socket: RawClient) {
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

pub fn on_friend_request_denied(payload: Payload, _socket: RawClient) {
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

pub fn on_unfriended(payload: Payload, _socket: RawClient) {
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

pub fn on_friend_cursor_position(payload: Payload, _socket: RawClient) {
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

pub fn on_friend_disconnected(payload: Payload, _socket: RawClient) {
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

pub fn on_friend_doll_created(payload: Payload, _socket: RawClient) {
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

pub fn on_friend_doll_updated(payload: Payload, _socket: RawClient) {
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

pub fn on_friend_doll_deleted(payload: Payload, _socket: RawClient) {
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

pub fn on_friend_active_doll_changed(payload: Payload, _socket: RawClient) {
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
