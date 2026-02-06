use rust_socketio::Payload;
use serde::de::DeserializeOwned;
use tracing::error;

/// Result type for payload operations
pub type PayloadResult<T> = Result<T, PayloadError>;

/// Errors that can occur during payload handling
#[derive(Debug)]
pub enum PayloadError {
    InvalidFormat,
    EmptyPayload,
    ParseError(()),
}

/// Extract the first value from a Text payload
pub fn extract_text_value(payload: Payload, event_name: &str) -> PayloadResult<serde_json::Value> {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                Ok(first_value.clone())
            } else {
                Err(PayloadError::EmptyPayload)
            }
        }
        _ => {
            error!("Received unexpected payload format for {}", event_name);
            Err(PayloadError::InvalidFormat)
        }
    }
}

/// Parse payload value into a specific type
pub fn parse_payload<T: DeserializeOwned>(
    value: serde_json::Value,
    event_name: &str,
) -> PayloadResult<T> {
    serde_json::from_value(value).map_err(|e| {
        error!("Failed to parse {} payload: {}", event_name, e);
        PayloadError::ParseError(())
    })
}

/// Extract and parse payload in one step
pub fn extract_and_parse<T: DeserializeOwned>(
    payload: Payload,
    event_name: &str,
) -> PayloadResult<T> {
    let value = extract_text_value(payload, event_name)?;
    parse_payload(value, event_name)
}

/// Check if a doll ID matches the current user's active doll
pub fn is_active_doll(doll_id: &str) -> bool {
    use crate::{lock_r, state::FDOLL};

    let guard = lock_r!(FDOLL);
    guard
        .user_data
        .user
        .as_ref()
        .and_then(|u| u.active_doll_id.as_ref())
        .map(|active_id| active_id == doll_id)
        .unwrap_or(false)
}

/// Extract doll ID from a JSON value
pub fn extract_doll_id(value: &serde_json::Value) -> Option<String> {
    value.get("id").and_then(|v| v.as_str()).map(String::from)
}
