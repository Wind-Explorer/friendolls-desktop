use rust_socketio::{Payload, RawClient};
use tracing::info;

use crate::services::app_events::AppEvents;
use crate::services::cursor::{normalized_to_absolute, CursorPositions};
use crate::state::AppDataRefreshScope;

use super::{
    emitter, refresh,
    types::{IncomingFriendCursorPayload, OutgoingFriendCursorPayload},
    utils,
};

/// Handler for friend-request-received event
pub fn on_friend_request_received(payload: Payload, _socket: RawClient) {
    if let Ok(value) = utils::extract_text_value(payload, "friend-request-received") {
        emitter::emit_to_frontend(AppEvents::FriendRequestReceived.as_str(), value);
    }
}

/// Handler for friend-request-accepted event
pub fn on_friend_request_accepted(payload: Payload, _socket: RawClient) {
    if let Ok(value) = utils::extract_text_value(payload, "friend-request-accepted") {
        emitter::emit_to_frontend(AppEvents::FriendRequestAccepted.as_str(), value);
        refresh::refresh_app_data(AppDataRefreshScope::Friends);
    }
}

/// Handler for friend-request-denied event
pub fn on_friend_request_denied(payload: Payload, _socket: RawClient) {
    if let Ok(value) = utils::extract_text_value(payload, "friend-request-denied") {
        emitter::emit_to_frontend(AppEvents::FriendRequestDenied.as_str(), value);
    }
}

/// Handler for unfriended event
pub fn on_unfriended(payload: Payload, _socket: RawClient) {
    if let Ok(value) = utils::extract_text_value(payload, "unfriended") {
        emitter::emit_to_frontend(AppEvents::Unfriended.as_str(), value);
        refresh::refresh_app_data(AppDataRefreshScope::Friends);
    }
}

/// Handler for friend-cursor-position event
pub fn on_friend_cursor_position(payload: Payload, _socket: RawClient) {
    if let Ok(friend_data) =
        utils::extract_and_parse::<IncomingFriendCursorPayload>(payload, "friend-cursor-position")
    {
        let mapped_pos = &friend_data.position;
        let raw_pos = normalized_to_absolute(mapped_pos);

        let outgoing_payload = OutgoingFriendCursorPayload {
            user_id: friend_data.user_id,
            position: CursorPositions {
                raw: raw_pos,
                mapped: mapped_pos.clone(),
            },
        };

        emitter::emit_to_frontend(AppEvents::FriendCursorPosition.as_str(), outgoing_payload);
    }
}

/// Handler for friend-disconnected event
pub fn on_friend_disconnected(payload: Payload, _socket: RawClient) {
    if let Ok(value) = utils::extract_text_value(payload, "friend-disconnected") {
        emitter::emit_to_frontend(AppEvents::FriendDisconnected.as_str(), value);
    }
}

/// Handler for friend-doll-created event
pub fn on_friend_doll_created(payload: Payload, _socket: RawClient) {
    handle_friend_doll_change("friend-doll-created", payload);
}

/// Handler for friend-doll-updated event
pub fn on_friend_doll_updated(payload: Payload, _socket: RawClient) {
    handle_friend_doll_change("friend-doll-updated", payload);
}

/// Handler for friend-doll-deleted event
pub fn on_friend_doll_deleted(payload: Payload, _socket: RawClient) {
    handle_friend_doll_change("friend-doll-deleted", payload);
}

/// Common handler for friend doll change events
fn handle_friend_doll_change(event_name: &str, payload: Payload) {
    if let Ok(value) = utils::extract_text_value(payload, event_name) {
        info!("Friend doll changed: {:?}", value);
        // Future: Could emit to frontend or trigger specific actions
    }
}

/// Handler for friend-active-doll-changed event
pub fn on_friend_active_doll_changed(payload: Payload, _socket: RawClient) {
    if let Ok(value) = utils::extract_text_value(payload, "friend-active-doll-changed") {
        emitter::emit_to_frontend(AppEvents::FriendActiveDollChanged.as_str(), value);
        refresh::refresh_app_data(AppDataRefreshScope::Friends);
    }
}

/// Handler for friend-user-status event
pub fn on_friend_user_status(payload: Payload, _socket: RawClient) {
    if let Ok(value) = utils::extract_text_value(payload, "friend-user-status") {
        emitter::emit_to_frontend(AppEvents::FriendUserStatus.as_str(), value);
    }
}
