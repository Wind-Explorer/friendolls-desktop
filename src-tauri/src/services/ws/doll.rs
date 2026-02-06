use rust_socketio::{Payload, RawClient};

use super::{refresh, utils};

/// Handler for doll.created event
pub fn on_doll_created(payload: Payload, _socket: RawClient) {
    if utils::extract_text_value(payload, "doll.created").is_ok() {
        refresh::refresh_app_data(crate::state::AppDataRefreshScope::Dolls);
    }
}

/// Handler for doll.updated event
pub fn on_doll_updated(payload: Payload, _socket: RawClient) {
    if let Ok(value) = utils::extract_text_value(payload, "doll.updated") {
        let doll_id = utils::extract_doll_id(&value);
        refresh::refresh_with_active_doll_check(doll_id.as_deref());
    }
}

/// Handler for doll.deleted event
pub fn on_doll_deleted(payload: Payload, _socket: RawClient) {
    if let Ok(value) = utils::extract_text_value(payload, "doll.deleted") {
        let doll_id = utils::extract_doll_id(&value);
        refresh::refresh_with_active_doll_check(doll_id.as_deref());
    }
}
