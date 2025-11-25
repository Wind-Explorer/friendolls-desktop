use device_query::{DeviceEvents, DeviceEventsHandler};
use serde::Serialize;
use std::time::Duration;
use tauri::ipc::Channel;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorPosition {
    pub x: i32,
    pub y: i32,
}

#[tauri::command]
pub async fn stream_cursor_position(on_event: Channel<CursorPosition>) {
    let device_state =
        DeviceEventsHandler::new(Duration::from_millis(200)).expect("Failed to start event loop");

    let _guard = device_state.on_mouse_move(move |position| {
        let pos = CursorPosition {
            x: position.0,
            y: position.1,
        };
        // Ignore send error (e.g. frontend closed channel)
        let _ = on_event.send(pos);
    });

    // Prevent function from exiting
    loop {
        tokio::time::sleep(Duration::ZERO).await;
    }
}
