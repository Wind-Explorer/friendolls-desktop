use device_query::{DeviceEvents, DeviceEventsHandler};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::{
    lock_r,
    services::{neko_positions, ws::report_cursor_data},
    state::FDOLL,
};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CursorPositions {
    pub raw: CursorPosition,
    pub mapped: CursorPosition,
}

static CURSOR_TRACKER: OnceCell<()> = OnceCell::new();
static LATEST_CURSOR_POSITION: OnceCell<Arc<Mutex<Option<CursorPosition>>>> = OnceCell::new();

/// Get the latest known cursor position (thread-safe)
pub fn get_latest_cursor_position() -> Option<CursorPosition> {
    LATEST_CURSOR_POSITION
        .get()
        .and_then(|mutex| mutex.lock().ok())
        .and_then(|guard| guard.clone())
}

/// Convert absolute screen coordinates to normalized coordinates (0.0 - 1.0)
pub fn absolute_to_normalized(pos: &CursorPosition) -> CursorPosition {
    let guard = lock_r!(FDOLL);
    let screen_w = guard.user_data.scene.display.screen_width as f64;
    let screen_h = guard.user_data.scene.display.screen_height as f64;

    CursorPosition {
        x: (pos.x / screen_w).clamp(0.0, 1.0),
        y: (pos.y / screen_h).clamp(0.0, 1.0),
    }
}

/// Convert normalized coordinates to absolute screen coordinates
pub fn normalized_to_absolute(normalized: &CursorPosition) -> CursorPosition {
    let guard = lock_r!(FDOLL);
    let screen_w = guard.user_data.scene.display.screen_width as f64;
    let screen_h = guard.user_data.scene.display.screen_height as f64;

    CursorPosition {
        x: (normalized.x * screen_w).round(),
        y: (normalized.y * screen_h).round(),
    }
}

/// Initialize cursor tracking.
pub async fn init_cursor_tracking() {
    info!("start_cursor_tracking called");

    // Use OnceCell to ensure this only runs once, even if called from multiple windows
    CURSOR_TRACKER.get_or_init(|| {
        // Initialize the shared state
        LATEST_CURSOR_POSITION.get_or_init(|| Arc::new(Mutex::new(None)));

        info!("First call to start_cursor_tracking - spawning cursor tracking task");
        tauri::async_runtime::spawn(async {
            if let Err(e) = init_cursor_tracking_i().await {
                error!("Failed to initialize cursor tracking: {}", e);
            }
        });
    });

    info!("Cursor tracking initialization registered");
}

async fn init_cursor_tracking_i() -> Result<(), String> {
    info!("Initializing cursor tracking...");

    // Create a channel to decouple event generation (producer) from processing (consumer).
    // Capacity 100 is plenty for 500ms polling (2Hz).
    let (tx, mut rx) = mpsc::channel::<CursorPositions>(100);

    // Spawn the consumer task
    // This task handles WebSocket reporting and local position projection updates.
    // It runs independently of the device event loop.
    tauri::async_runtime::spawn(async move {
        info!("Cursor event consumer started");

        while let Some(positions) = rx.recv().await {
            let mapped_for_ws = positions.mapped.clone();

            // 1. WebSocket reporting
            report_cursor_data(mapped_for_ws).await;

            // 2. Update unified neko positions projection
            neko_positions::update_self_cursor(positions);
        }
        warn!("Cursor event consumer stopped (channel closed)");
    });

    // Try to initialize the device event handler
    // Using 500ms sleep as requested by user to reduce CPU usage
    let device_state = DeviceEventsHandler::new(Duration::from_millis(500))
        .ok_or("Failed to create device event handler (already running?)")?;

    info!("Device event handler created successfully");
    info!("Setting up mouse move handler for event broadcasting...");

    // Get scale factor from global state
    #[cfg(target_os = "windows")]
    let scale_factor = {
        let guard = lock_r!(FDOLL);
        guard.user_data.scene.display.monitor_scale_factor
    };

    // The producer closure moves `tx` into it.
    // device_query runs this closure on its own thread.
    let _guard = device_state.on_mouse_move(move |position: &(i32, i32)| {
        // `device_query` crate appears to behave
        // differently on Windows vs other platforms.
        //
        // It doesn't take into account the monitor scale
        // factor on Windows, so we handle it manually.
        #[cfg(target_os = "windows")]
        let raw = CursorPosition {
            x: position.0 as f64 / scale_factor,
            y: position.1 as f64 / scale_factor,
        };

        #[cfg(not(target_os = "windows"))]
        let raw = CursorPosition {
            x: position.0 as f64,
            y: position.1 as f64,
        };

        // Update global state
        if let Some(mutex) = LATEST_CURSOR_POSITION.get() {
            if let Ok(mut guard) = mutex.lock() {
                *guard = Some(raw.clone());
            }
        }

        let mapped = absolute_to_normalized(&raw);

        let positions = CursorPositions { raw, mapped };

        // Send to consumer channel (non-blocking)
        if let Err(e) = tx.try_send(positions) {
            debug!("Failed to send cursor position to channel: {:?}", e);
        }
    });

    info!("Mouse move handler registered - now broadcasting cursor events to all windows");

    // Keep the handler alive forever
    // This loop is necessary to keep `_guard` and `device_state` in scope.
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}
