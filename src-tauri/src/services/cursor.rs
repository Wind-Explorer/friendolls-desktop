use device_query::{DeviceEvents, DeviceEventsHandler};
use serde::Serialize;
use std::time::Duration;
use tauri::ipc::Channel;

use crate::get_app_handle;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorPositions {
    pub raw: CursorPosition,
    pub mapped: CursorPosition,
}

fn map_to_grid(
    pos: &CursorPosition,
    grid_size: i32,
    screen_w: i32,
    screen_h: i32,
) -> CursorPosition {
    CursorPosition {
        x: pos.x * grid_size / screen_w,
        y: pos.y * grid_size / screen_h,
    }
}

#[tauri::command]
pub async fn channel_cursor_positions(on_event: Channel<CursorPositions>) {
    let app_handle = get_app_handle();
    let primary_monitor = app_handle.primary_monitor().unwrap().unwrap();
    let monitor_dimensions = primary_monitor.size();
    let logical_monitor_dimensions: tauri::LogicalSize<i32> =
        monitor_dimensions.to_logical(primary_monitor.scale_factor());
    let device_state =
        DeviceEventsHandler::new(Duration::from_millis(200)).expect("Failed to start event loop");

    let _guard = device_state.on_mouse_move(move |position| {
        let raw = CursorPosition {
            x: position.0,
            y: position.1,
        };
        let mapped = map_to_grid(
            &raw,
            600,
            logical_monitor_dimensions.width,
            logical_monitor_dimensions.height,
        );
        let positions = CursorPositions { raw, mapped };
        let _ = on_event.send(positions);
    });

    loop {
        // for whatever reason this sleep is not taking effect but it
        // does reduce CPU usage on my Mac from 100% to 6% so...cool!
        tokio::time::sleep(Duration::from_millis(1000)).await
    }
}
