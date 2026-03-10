use crate::{
    get_app_handle, lock_r, lock_w,
    remotes::{dolls::DollsRemote, friends::FriendRemote, user::UserRemote},
    services::{
        app_events::{ActiveDollSpriteChanged, AppDataRefreshed},
        friend_active_doll_sprite, friend_cursor, sprite,
    },
    state::FDOLL,
};
use std::{collections::HashSet, sync::LazyLock};
use tauri_specta::Event as _;
use tokio::sync::Mutex;
use tracing::{info, warn};

pub fn update_display_dimensions_for_scene_state() {
    let app_handle = get_app_handle();

    let mut guard = lock_w!(FDOLL);

    // Get primary monitor with retries
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

        guard.user_data.scene.display.screen_width = logical_monitor_dimensions.width;
        guard.user_data.scene.display.screen_height = logical_monitor_dimensions.height;
        guard.user_data.scene.display.monitor_scale_factor = monitor_scale_factor;
        guard.user_data.scene.grid_size = 600; // Hardcoded grid size

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

static REFRESH_IN_FLIGHT: LazyLock<Mutex<HashSet<AppDataRefreshScope>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static REFRESH_PENDING: LazyLock<Mutex<HashSet<AppDataRefreshScope>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

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
                        let mut guard = lock_w!(crate::state::FDOLL);
                        guard.user_data.user = Some(user);
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
                        let mut guard = lock_w!(crate::state::FDOLL);
                        guard.user_data.friends = Some(friends);
                        drop(guard);
                        friend_active_doll_sprite::sync_from_app_data();
                        friend_cursor::sync_from_app_data();
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
                        let mut guard = lock_w!(crate::state::FDOLL);
                        guard.user_data.dolls = Some(dolls);
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
                let guard = lock_r!(crate::state::FDOLL); // Use read lock to get data
                let app_data_clone = guard.user_data.clone();
                drop(guard); // Drop lock before emitting to prevent potential deadlocks

                if let Err(e) = AppDataRefreshed(app_data_clone).emit(get_app_handle()) {
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

                if matches!(
                    scope,
                    AppDataRefreshScope::All
                        | AppDataRefreshScope::User
                        | AppDataRefreshScope::Dolls
                ) {
                    match sprite::get_active_doll_sprite_base64() {
                        Ok(sprite_b64) => {
                            if let Err(e) =
                                ActiveDollSpriteChanged(sprite_b64).emit(get_app_handle())
                            {
                                warn!("Failed to emit active-doll-sprite-changed event: {}", e);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to generate active doll sprite: {}", e);
                        }
                    }
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

pub fn clear_app_data() {
    let mut guard = lock_w!(FDOLL);
    guard.user_data.dolls = None;
    guard.user_data.user = None;
    guard.user_data.friends = None;
    drop(guard);
    friend_active_doll_sprite::clear();
    friend_cursor::clear();
}
