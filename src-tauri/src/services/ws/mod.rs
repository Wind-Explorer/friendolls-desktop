use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)] // pretend to be a const like in js
pub struct WS_EVENT;

#[derive(Debug, Deserialize)]
pub struct IncomingFriendCursorPayload {
    #[serde(rename = "userId")]
    user_id: String,
    position: crate::services::cursor::CursorPosition,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutgoingFriendCursorPayload {
    user_id: String,
    position: crate::services::cursor::CursorPositions,
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

mod client;
mod connection;
mod cursor;
mod doll;
mod friend;
mod handlers;
mod interaction;

pub use client::init_ws_client;
pub use cursor::report_cursor_data;
