use rust_socketio::{ClientBuilder, Payload, RawClient};
use serde_json::json;
use tauri::async_runtime;
use tracing::error;

use crate::{
    core::{models::app_config::AppConfig, state::FDOLL},
    lock_r,
    services::cursor::CursorPosition,
};

// Define a callback for handling incoming messages (e.g., 'pong')
fn on_pong(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(str) => println!("Received pong: {:?}", str),
        Payload::Binary(bin) => println!("Received pong (binary): {:?}", bin),
        _ => todo!(),
    }
}

pub async fn report_cursor_data(cursor_position: CursorPosition) {
    let client = {
        let guard = lock_r!(FDOLL);
        guard
            .clients
            .as_ref()
            .expect("Clients are initialized")
            .ws_client
            .as_ref()
            .expect("WebSocket client is initialized")
            .clone()
    };

    match async_runtime::spawn_blocking(move || {
        client.emit(
            "cursor-report-position",
            Payload::Text(vec![json!({ "position": cursor_position })]),
        )
    })
    .await
    {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => error!("Failed to emit ping: {}", e),
        Err(e) => error!("Failed to execute blocking task: {}", e),
    }
}

pub fn build_ws_client(app_config: &AppConfig) -> rust_socketio::client::Client {
    let client = match ClientBuilder::new(
        app_config
            .api_base_url
            .as_ref()
            .expect("Missing API base URL"),
    )
    .namespace("/")
    .on("pong", on_pong)
    .connect()
    {
        Ok(c) => c,
        Err(_) => todo!("TODO error handling"),
    };
    client
}
