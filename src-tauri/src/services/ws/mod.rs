/// WebSocket module for real-time communication
///
/// Organized into focused submodules:
/// - types: Event constants and payload structures
/// - utils: Common payload handling and parsing utilities
/// - emitter: WebSocket and frontend event emission
/// - refresh: Data refresh orchestration
/// - handlers: Event handler registration
/// - connection: Connection lifecycle handlers
/// - doll: Doll-related event handlers
/// - friend: Friend-related event handlers
/// - interaction: Interaction event handlers
/// - cursor: Cursor position reporting
/// - user_status: User status reporting
mod connection;
mod cursor;
mod doll;
mod emitter;
mod friend;
mod handlers;
mod interaction;
mod refresh;
mod types;
mod user_status;
mod utils;

pub mod client;

// Re-export public API
pub use cursor::report_cursor_data;
pub use types::WS_EVENT;
pub use user_status::{report_user_status, UserStatusPayload};
