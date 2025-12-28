// in app-core/src/state.rs
use crate::{
    get_app_handle, lock_r, lock_w,
    models::{
        app_config::{AppConfig, AuthConfig},
        app_data::AppData,
    },
    remotes::{dolls::DollsRemote, friends::FriendRemote, user::UserRemote},
    services::auth::{load_auth_pass, AuthPass},
};
use std::{
    collections::HashSet,
    env,
    sync::{Arc, LazyLock, RwLock},
};
use tauri::Emitter;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Default, Clone)]
pub struct OAuthFlowTracker {
    pub state: Option<String>,
    pub code_verifier: Option<String>,
    pub initiated_at: Option<u64>,
    pub cancel_token: Option<tokio_util::sync::CancellationToken>,
}

pub struct Clients {
    pub http_client: reqwest::Client,
    pub ws_client: Option<rust_socketio::client::Client>,
    pub is_ws_initialized: bool,
}

#[derive(Default)]
pub struct AppState {
    pub app_config: AppConfig,
    pub clients: Option<Clients>,
    pub auth_pass: Option<AuthPass>,
    pub oauth_flow: OAuthFlowTracker,
    pub tracing_guard: Option<tracing_appender::non_blocking::WorkerGuard>,

    // exposed to the frontend
    pub app_data: AppData,
}

// Global application state
// Read / write this state via the `lock_r!` / `lock_w!` macros from `fdoll-core::utilities`
pub static FDOLL: LazyLock<Arc<RwLock<AppState>>> =
    LazyLock::new(|| Arc::new(RwLock::new(AppState::default())));

pub fn init_fdoll_state(tracing_guard: Option<tracing_appender::non_blocking::WorkerGuard>) {
    {
        let mut guard = lock_w!(FDOLL);
        dotenvy::dotenv().ok();
        guard.tracing_guard = tracing_guard;
        guard.app_config = AppConfig {
            api_base_url: Some(env::var("API_BASE_URL").expect("API_BASE_URL must be set")),
            auth: AuthConfig {
                audience: env::var("JWT_AUDIENCE").expect("JWT_AUDIENCE must be set"),
                auth_url: env::var("AUTH_URL").expect("AUTH_URL must be set"),
            },
        };
        guard.auth_pass = match load_auth_pass() {
            Ok(pass) => pass,
            Err(e) => {
                warn!("Failed to load auth pass from keyring: {e}");
                None
            }
        };
        info!("Loaded auth pass");

        // Initialize HTTP client immediately (non-blocking)
        let http_client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .user_agent("friendolls-desktop/0.1.0")
            .build()
            .expect("Client should build");

        // Store HTTP client immediately - WebSocket client will be added later
        guard.clients = Some(Clients {
            http_client,
            ws_client: None,
            is_ws_initialized: false,
        });
        info!("Initialized HTTP client");

        // Initialize screen dimensions
        let app_handle = get_app_handle();

        // Get primary monitor with retries
        // Note: This duplicates logic from init_cursor_tracking, but we need it here for global state
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
                    Err(e) => {
                        retry_count += 1;
                        if retry_count >= max_retries {
                            warn!("Failed to get primary monitor during state init: {}", e);
                            break None;
                        }
                        warn!(
                            "Error getting primary monitor during state init, retrying... ({}/{}): {}",
                            retry_count, max_retries, e
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

            guard.app_data.scene.display.screen_width = logical_monitor_dimensions.width;
            guard.app_data.scene.display.screen_height = logical_monitor_dimensions.height;
            guard.app_data.scene.display.monitor_scale_factor = monitor_scale_factor;
            guard.app_data.scene.grid_size = 600; // Hardcoded grid size

            info!(
                "Initialized global AppData with screen dimensions: {}x{}, scale: {}, grid: {}",
                logical_monitor_dimensions.width,
                logical_monitor_dimensions.height,
                monitor_scale_factor,
                guard.app_data.scene.grid_size
            );
        } else {
            warn!("Could not initialize screen dimensions in global state - no monitor found");
        }
    }

    info!("Initialized FDOLL state (WebSocket client & user data initializing asynchronously)");
}

/// Defines which parts of AppData should be refreshed from the server
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppDataRefreshScope {
    /// Refresh all data (user profile + friends list + dolls list)
    All,
    /// Refresh only user profile
    User,
    /// Refresh only friends list
    Friends,
    /// Refresh only dolls list
    Dolls,
}

/// To be called in init state or need to refresh data.
/// Populate user data in app state from the server.
///
/// This is a convenience wrapper that refreshes all data.
/// For more control, use `init_app_data_scoped`.
pub async fn init_app_data() {
    init_app_data_scoped(AppDataRefreshScope::All).await;
}

/// Populate specific parts of app data from the server based on the scope.
///
/// # Arguments
/// * `scope` - Determines which data to refresh (All, User, or Friends)
///
/// # Examples
/// ```
/// // Refresh only friends list when a friend request is accepted
/// init_app_data_scoped(AppDataRefreshScope::Friends).await;
///
/// // Refresh only user profile when updating user settings
/// init_app_data_scoped(AppDataRefreshScope::User).await;
/// ```
static REFRESH_IN_FLIGHT: LazyLock<Mutex<HashSet<AppDataRefreshScope>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static REFRESH_PENDING: LazyLock<Mutex<HashSet<AppDataRefreshScope>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

pub async fn init_app_data_scoped(scope: AppDataRefreshScope) {
    loop {
        // Deduplicate concurrent refreshes for the same scope
        {
            let mut in_flight = REFRESH_IN_FLIGHT.lock().await;
            if in_flight.contains(&scope) {
                let mut pending = REFRESH_PENDING.lock().await;
                pending.insert(scope);
                return;
            }
            in_flight.insert(scope);
        }

        let result: Result<(), ()> = async {
            let user_remote = UserRemote::new();
            let friend_remote = FriendRemote::new();
            let dolls_remote = DollsRemote::new();

            // Fetch user profile if needed
            if matches!(scope, AppDataRefreshScope::All | AppDataRefreshScope::User) {
                match user_remote.get_user(None).await {
                    Ok(user) => {
                        let mut guard = lock_w!(FDOLL);
                        guard.app_data.user = Some(user);
                    }
                    Err(e) => {
                        warn!("Failed to fetch user profile: {}", e);
                        use tauri_plugin_dialog::MessageDialogBuilder;
                        use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

                        let handle = get_app_handle();
                        MessageDialogBuilder::new(
                            handle.dialog().clone(),
                            "Network Error",
                            "Failed to fetch user profile. You may be offline.",
                        )
                        .kind(MessageDialogKind::Error)
                        .show(|_| {});
                        return Err(());
                    }
                }
            }

            // Fetch friends list if needed
            if matches!(
                scope,
                AppDataRefreshScope::All | AppDataRefreshScope::Friends
            ) {
                match friend_remote.get_friends().await {
                    Ok(friends) => {
                        let mut guard = lock_w!(FDOLL);
                        guard.app_data.friends = Some(friends);
                    }
                    Err(e) => {
                        warn!("Failed to fetch friends list: {}", e);
                        use tauri_plugin_dialog::MessageDialogBuilder;
                        use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

                        let handle = get_app_handle();
                        MessageDialogBuilder::new(
                            handle.dialog().clone(),
                            "Network Error",
                            "Failed to fetch friends list. You may be offline.",
                        )
                        .kind(MessageDialogKind::Error)
                        .show(|_| {});
                        return Err(());
                    }
                }
            }

            // Fetch dolls list if needed
            if matches!(scope, AppDataRefreshScope::All | AppDataRefreshScope::Dolls) {
                match dolls_remote.get_dolls().await {
                    Ok(dolls) => {
                        let mut guard = lock_w!(FDOLL);
                        guard.app_data.dolls = Some(dolls);
                    }
                    Err(e) => {
                        warn!("Failed to fetch dolls list: {}", e);
                        use tauri_plugin_dialog::MessageDialogBuilder;
                        use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

                        let handle = get_app_handle();
                        MessageDialogBuilder::new(
                            handle.dialog().clone(),
                            "Network Error",
                            "Failed to fetch dolls list. You may be offline.",
                        )
                        .kind(MessageDialogKind::Error)
                        .show(|_| {});
                        return Err(());
                    }
                }
            }

            // Emit event regardless of partial success, frontend should handle nulls/empty states
            {
                let guard = lock_r!(FDOLL); // Use read lock to get data
                let app_data_clone = guard.app_data.clone();
                drop(guard); // Drop lock before emitting to prevent potential deadlocks

                if let Err(e) = get_app_handle().emit("app-data-refreshed", &app_data_clone) {
                    warn!("Failed to emit app-data-refreshed event: {}", e);
                    use tauri_plugin_dialog::MessageDialogBuilder;
                    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

                    let handle = get_app_handle();
                    MessageDialogBuilder::new(
                        handle.dialog().clone(),
                        "Sync Error",
                        "Could not broadcast refreshed data to the UI. Some data may be stale.",
                    )
                    .kind(MessageDialogKind::Error)
                    .show(|_| {});
                }
            }

            Ok(())
        }
        .await;

        // Clear in-flight marker even on early exit
        {
            let mut in_flight = REFRESH_IN_FLIGHT.lock().await;
            in_flight.remove(&scope);
        }

        // If a refresh was queued while this one was running, run again
        let rerun = {
            let mut pending = REFRESH_PENDING.lock().await;
            pending.remove(&scope)
        };

        if rerun {
            continue;
        }

        if result.is_err() {
            return;
        }

        break;
    }
}
