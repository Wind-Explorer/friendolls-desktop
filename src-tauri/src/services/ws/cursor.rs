use crate::services::cursor::CursorPosition;

use super::{emitter, types::WS_EVENT};

/// Report cursor position to WebSocket server
///
/// Uses soft emit since cursor telemetry is non-critical
/// and should not tear down the session on failure.
pub async fn report_cursor_data(cursor_position: CursorPosition) {
    let _ = emitter::ws_emit_soft(WS_EVENT::CURSOR_REPORT_POSITION, cursor_position).await;
}
