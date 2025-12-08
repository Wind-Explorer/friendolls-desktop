use rust_socketio::{ClientBuilder, Payload, RawClient};
use serde_json::json;
use tauri::async_runtime;
use tracing::{error, info};

use crate::{
    lock_r, lock_w,
    services::cursor::CursorPosition,
    {models::app_config::AppConfig, state::FDOLL},
};

#[allow(non_camel_case_types)] // pretend to be a const like in js
pub struct WS_EVENT;

impl WS_EVENT {
    pub const CURSOR_REPORT_POSITION: &str = "cursor-report-position";
}

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
            WS_EVENT::CURSOR_REPORT_POSITION,
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

pub async fn init_ws_client() {
    let app_config = {
        let guard = lock_r!(FDOLL);
        guard.app_config.clone()
    };

    let ws_client = build_ws_client(&app_config).await;

    let mut guard = lock_w!(FDOLL);
    if let Some(clients) = guard.clients.as_mut() {
        clients.ws_client = Some(ws_client);
    }
    info!("WebSocket client initialized after authentication");
}

pub async fn build_ws_client(app_config: &AppConfig) -> rust_socketio::client::Client {
    let token = crate::services::auth::get_access_token()
        .await
        .expect("No access token available for WebSocket connection");

    info!("Building WebSocket client with authentication");

    let api_base_url = app_config
        .api_base_url
        .clone()
        .expect("Missing API base URL");

    let client = async_runtime::spawn_blocking(move || {
        ClientBuilder::new(api_base_url)
            .namespace("/")
            .on("pong", on_pong)
            .auth(json!({ "token": token }))
            .connect()
    })
    .await
    .expect("Failed to spawn blocking task");

    match client {
        Ok(c) => {
            info!("WebSocket client connected successfully");
            c
        }
        Err(e) => {
            error!("Failed to connect WebSocket: {:?}", e);
            panic!(
                "TODO: better error handling for WebSocket connection - {}",
                e
            );
        }
    }
}
