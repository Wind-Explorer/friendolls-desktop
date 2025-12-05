use device_query::{DeviceEvents, DeviceEventsHandler};
use once_cell::sync::OnceCell;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tracing::{error, info, warn};
use ts_rs::TS;

use crate::get_app_handle;

#[derive(Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CursorPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CursorPositions {
    pub raw: CursorPosition,
    pub mapped: CursorPosition,
}

static CURSOR_TRACKER: OnceCell<()> = OnceCell::new();

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

/// Initialize cursor tracking - can be called multiple times safely from any window
/// Only the first call will actually start tracking, subsequent calls are no-ops
#[tauri::command]
pub async fn start_cursor_tracking() -> Result<(), String> {
    info!("start_cursor_tracking called");

    // Use OnceCell to ensure this only runs once, even if called from multiple windows
    CURSOR_TRACKER.get_or_init(|| {
        info!("First call to start_cursor_tracking - spawning cursor tracking task");
        tauri::async_runtime::spawn(async {
            if let Err(e) = init_cursor_tracking().await {
                error!("Failed to initialize cursor tracking: {}", e);
            }
        });
    });

    info!("Cursor tracking initialization registered");
    Ok(())
}

async fn init_cursor_tracking() -> Result<(), String> {
    info!("Initializing cursor tracking...");
    let app_handle = get_app_handle();

    // Get primary monitor with retries
    let primary_monitor = {
        let mut retry_count = 0;
        let max_retries = 3;
        loop {
            match app_handle.primary_monitor() {
                Ok(Some(monitor)) => {
                    info!("Primary monitor acquired");
                    break monitor;
                }
                Ok(None) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        return Err(format!(
                            "No primary monitor found after {} retries",
                            max_retries
                        ));
                    }
                    warn!(
                        "Primary monitor not available, retrying... ({}/{})",
                        retry_count, max_retries
                    );
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        return Err(format!("Failed to get primary monitor: {}", e));
                    }
                    warn!(
                        "Error getting primary monitor, retrying... ({}/{}): {}",
                        retry_count, max_retries, e
                    );
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    };

    let monitor_dimensions = primary_monitor.size();
    let logical_monitor_dimensions: tauri::LogicalSize<i32> =
        monitor_dimensions.to_logical(primary_monitor.scale_factor());

    info!(
        "Monitor dimensions: {}x{}",
        logical_monitor_dimensions.width, logical_monitor_dimensions.height
    );

    // Try to initialize the device event handler
    let device_state = DeviceEventsHandler::new(Duration::from_millis(500))
        .ok_or("Failed to create device event handler (already running?)")?;

    info!("Device event handler created successfully");
    info!("Setting up mouse move handler for event broadcasting...");

    let send_count = Arc::new(AtomicU64::new(0));
    let send_count_clone = Arc::clone(&send_count);
    let app_handle_clone = app_handle.clone();

    let _guard = device_state.on_mouse_move(move |position: &(i32, i32)| {
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
        let positions = CursorPositions {
            raw,
            mapped: mapped.clone(),
        };

        // Report to server (existing functionality)
        let mapped_for_ws = mapped.clone();
        tauri::async_runtime::spawn(async move {
            crate::services::ws::report_cursor_data(mapped_for_ws).await;
        });

        // Broadcast to ALL windows using events
        match app_handle_clone.emit("cursor-position", &positions) {
            Ok(_) => {
                let count = send_count_clone.fetch_add(1, Ordering::Relaxed) + 1;
                if count % 100 == 0 {
                    info!("Broadcast {} cursor position updates to all windows. Latest: raw({}, {}), mapped({}, {})",
                          count, positions.raw.x, positions.raw.y, positions.mapped.x, positions.mapped.y);
                }
            }
            Err(e) => {
                error!("Failed to emit cursor position event: {:?}", e);
            }
        }
    });

    info!("Mouse move handler registered - now broadcasting cursor events to all windows");

    // Keep the handler alive forever
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}
