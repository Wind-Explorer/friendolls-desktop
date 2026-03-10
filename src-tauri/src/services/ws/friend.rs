use rust_socketio::{Payload, RawClient};
use tracing::info;

use crate::models::event_payloads::{
    FriendActiveDollChangedPayload, FriendDisconnectedPayload, FriendRequestAcceptedPayload,
    FriendRequestDeniedPayload, FriendRequestReceivedPayload, FriendUserStatusPayload,
    UnfriendedPayload,
};
use crate::services::app_events::{
    FriendActiveDollChanged, FriendDisconnected, FriendRequestAccepted, FriendRequestDenied,
    FriendRequestReceived, FriendUserStatusChanged, Unfriended,
};
use crate::services::{
    cursor::{normalized_to_absolute, CursorPositions},
    friend_active_doll_sprite, friend_cursor,
};
use crate::state::AppDataRefreshScope;

use super::{emitter, refresh, types::IncomingFriendCursorPayload, utils};

/// Handler for friend-request-received event
pub fn on_friend_request_received(payload: Payload, _socket: RawClient) {
    if let Ok(data) =
        utils::extract_and_parse::<FriendRequestReceivedPayload>(payload, "friend-request-received")
    {
        emitter::emit_to_frontend_typed(&FriendRequestReceived(data));
    }
}

/// Handler for friend-request-accepted event
pub fn on_friend_request_accepted(payload: Payload, _socket: RawClient) {
    if let Ok(data) =
        utils::extract_and_parse::<FriendRequestAcceptedPayload>(payload, "friend-request-accepted")
    {
        emitter::emit_to_frontend_typed(&FriendRequestAccepted(data));
        refresh::refresh_app_data(AppDataRefreshScope::Friends);
    }
}

/// Handler for friend-request-denied event
pub fn on_friend_request_denied(payload: Payload, _socket: RawClient) {
    if let Ok(data) =
        utils::extract_and_parse::<FriendRequestDeniedPayload>(payload, "friend-request-denied")
    {
        emitter::emit_to_frontend_typed(&FriendRequestDenied(data));
    }
}

/// Handler for unfriended event
pub fn on_unfriended(payload: Payload, _socket: RawClient) {
    if let Ok(data) = utils::extract_and_parse::<UnfriendedPayload>(payload, "unfriended") {
        emitter::emit_to_frontend_typed(&Unfriended(data));
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

        friend_cursor::update_position(
            friend_data.user_id,
            CursorPositions {
                raw: raw_pos,
                mapped: mapped_pos.clone(),
            },
        );
    }
}

/// Handler for friend-disconnected event
pub fn on_friend_disconnected(payload: Payload, _socket: RawClient) {
    if let Ok(data) =
        utils::extract_and_parse::<FriendDisconnectedPayload>(payload, "friend-disconnected")
    {
        friend_active_doll_sprite::remove_friend(&data.user_id);
        friend_cursor::remove_friend(&data.user_id);
        emitter::emit_to_frontend_typed(&FriendDisconnected(data));
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
    if let Ok(data) = utils::extract_and_parse::<FriendActiveDollChangedPayload>(
        payload,
        "friend-active-doll-changed",
    ) {
        friend_active_doll_sprite::set_active_doll(&data.friend_id, data.doll.as_ref());
        friend_cursor::set_active_doll(&data.friend_id, data.doll.is_some());
        emitter::emit_to_frontend_typed(&FriendActiveDollChanged(data));
    }
}

/// Handler for friend-user-status event
pub fn on_friend_user_status(payload: Payload, _socket: RawClient) {
    if let Ok(data) =
        utils::extract_and_parse::<FriendUserStatusPayload>(payload, "friend-user-status")
    {
        if !data.status.has_presence_content() {
            return;
        }

        emitter::emit_to_frontend_typed(&FriendUserStatusChanged(data));
    }
}
