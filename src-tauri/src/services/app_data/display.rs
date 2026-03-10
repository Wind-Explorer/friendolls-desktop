use crate::{get_app_handle, lock_w, state::FDOLL};
use tracing::{info, warn};

pub fn update_display_dimensions_for_scene_state() {
    let app_handle = get_app_handle();

    let mut guard = lock_w!(FDOLL);

    let primary_monitor = {
        let mut retry_count = 0;
        let max_retries = 3;
        loop {
            match app_handle.primary_monitor() {
                Ok(Some(monitor)) => {
                    info!("Primary monitor acquired for state initialization");
                    break Some(monitor);
                }
                Ok(None) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        warn!(
                            "No primary monitor found after {} retries during state init",
                            max_retries
                        );
                        break None;
                    }
                    warn!(
                        "Primary monitor not available during state init, retrying... ({}/{})",
                        retry_count, max_retries
                    );
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(error) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        warn!("Failed to get primary monitor during state init: {}", error);
                        break None;
                    }
                    warn!(
                        "Error getting primary monitor during state init, retrying... ({}/{}): {}",
                        retry_count, max_retries, error
                    );
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    };

    if let Some(monitor) = primary_monitor {
        let monitor_dimensions = monitor.size();
        let monitor_scale_factor = monitor.scale_factor();
        let logical_monitor_dimensions: tauri::LogicalSize<i32> =
            monitor_dimensions.to_logical(monitor_scale_factor);

        guard.user_data.scene.display.screen_width = logical_monitor_dimensions.width;
        guard.user_data.scene.display.screen_height = logical_monitor_dimensions.height;
        guard.user_data.scene.display.monitor_scale_factor = monitor_scale_factor;
        guard.user_data.scene.grid_size = 600;

        info!(
            "Initialized global AppData with screen dimensions: {}x{}, scale: {}, grid: {}",
            logical_monitor_dimensions.width,
            logical_monitor_dimensions.height,
            monitor_scale_factor,
            guard.user_data.scene.grid_size
        );
    } else {
        warn!("Could not initialize screen dimensions in global state - no monitor found");
    }
}
