use rust_socketio::{ClientBuilder, Event};

use super::types::WS_EVENT;

pub fn register_event_handlers(builder: ClientBuilder) -> ClientBuilder {
    builder
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
        .on(
            WS_EVENT::FRIEND_USER_STATUS,
            super::friend::on_friend_user_status,
        )
        .on(WS_EVENT::DOLL_CREATED, super::doll::on_doll_created)
        .on(WS_EVENT::DOLL_UPDATED, super::doll::on_doll_updated)
        .on(WS_EVENT::DOLL_DELETED, super::doll::on_doll_deleted)
        .on(WS_EVENT::INITIALIZED, super::connection::on_initialized)
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
        .on(Event::Connect, super::connection::on_connected)
}
