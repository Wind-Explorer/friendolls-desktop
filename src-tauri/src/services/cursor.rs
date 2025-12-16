use device_query::{DeviceEvents, DeviceEventsHandler};
use once_cell::sync::OnceCell;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tracing::{error, info};
use ts_rs::TS;

use crate::{get_app_handle, lock_r, state::FDOLL};

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

/// Convert absolute screen coordinates to grid coordinates
pub fn absolute_position_to_grid(pos: &CursorPosition) -> CursorPosition {
    let guard = lock_r!(FDOLL);
    let grid_size = guard.app_data.scene.grid_size;
    let screen_w = guard.app_data.scene.display.screen_width;
    let screen_h = guard.app_data.scene.display.screen_height;

    CursorPosition {
        x: pos.x * grid_size / screen_w,
        y: pos.y * grid_size / screen_h,
    }
}

/// Convert grid coordinates to absolute screen coordinates
pub fn grid_to_absolute_position(grid: &CursorPosition) -> CursorPosition {
    let guard = lock_r!(FDOLL);
    let grid_size = guard.app_data.scene.grid_size;
    let screen_w = guard.app_data.scene.display.screen_width;
    let screen_h = guard.app_data.scene.display.screen_height;

    CursorPosition {
        x: (grid.x * screen_w + grid_size / 2) / grid_size,
        y: (grid.y * screen_h + grid_size / 2) / grid_size,
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

    // Try to initialize the device event handler
    let device_state = DeviceEventsHandler::new(Duration::from_millis(500))
        .ok_or("Failed to create device event handler (already running?)")?;

    info!("Device event handler created successfully");
    info!("Setting up mouse move handler for event broadcasting...");

    let send_count = Arc::new(AtomicU64::new(0));
    let send_count_clone = Arc::clone(&send_count);
    let app_handle_clone = app_handle.clone();

    #[cfg(target_os = "windows")]
    {
        // Get scale factor from global state
        let scale_factor = {
            let guard = lock_r!(FDOLL);
            guard.app_data.scene.display.monitor_scale_factor
        };
    }

    let _guard = device_state.on_mouse_move(move |position: &(i32, i32)| {

        // `device_query` crate appears to behave
        // differently on Windows vs other platforms.
        //
        // It doesn't take into account the monitor scale
        // factor on Windows, so we handle it manually.
        #[cfg(target_os = "windows")]
        let raw = CursorPosition {
            x: (position.0 as f64 / scale_factor) as i32,
            y: (position.1 as f64 / scale_factor) as i32,
        };

        #[cfg(not(target_os = "windows"))]
        let raw = CursorPosition {
            x: position.0,
            y: position.1,
        };

        let mapped = absolute_position_to_grid(&raw);
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
