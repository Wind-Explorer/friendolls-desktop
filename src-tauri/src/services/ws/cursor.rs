use crate::services::cursor::CursorPosition;

use super::{emitter, types::WS_EVENT};

/// Report cursor position to WebSocket server
pub async fn report_cursor_data(cursor_position: CursorPosition) {
    let _ = emitter::ws_emit(WS_EVENT::CURSOR_REPORT_POSITION, cursor_position).await;
}
