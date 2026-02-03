use tauri::Manager;
use tracing_subscriber::util::SubscriberInitExt;

use crate::get_app_handle;

/// Initialize `tracing_subscriber` for logging to file & console
pub fn init_logging() {
    // Set up file appender
    let app_handle = get_app_handle();
    let app_log_dir = app_handle
        .path()
        .app_log_dir()
        .expect("Could not determine app log dir");

    // Create the directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&app_log_dir) {
        eprintln!("Failed to create log directory: {}", e);
    }

    let file_appender = tracing_appender::rolling::daily(&app_log_dir, "friendolls.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Create a filter - adjust the level as needed (trace, debug, info, warn, error)
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    // Create a layer that writes to the file
    let file_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_writer(non_blocking);

    // Create a layer that writes to stdout (console)
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true);

    // Combine both layers with filter
    use tracing_subscriber::layer::SubscriberExt;
    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .init();
}
