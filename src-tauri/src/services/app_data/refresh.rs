use std::{collections::HashSet, sync::LazyLock};

use tauri_plugin_dialog::MessageDialogBuilder;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_specta::Event as _;
use tokio::sync::Mutex;
use tracing::warn;

use crate::{
    get_app_handle, lock_r, lock_w,
    remotes::{dolls::DollsRemote, friends::FriendRemote, user::UserRemote},
    services::{
        app_events::{ActiveDollSpriteChanged, AppDataRefreshed},
        friends, sprite,
    },
    state::FDOLL,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppDataRefreshScope {
    All,
    User,
    Friends,
    Dolls,
}

static REFRESH_IN_FLIGHT: LazyLock<Mutex<HashSet<AppDataRefreshScope>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static REFRESH_PENDING: LazyLock<Mutex<HashSet<AppDataRefreshScope>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

pub async fn init_app_data_scoped(scope: AppDataRefreshScope) {
    loop {
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

            if matches!(scope, AppDataRefreshScope::All | AppDataRefreshScope::User) {
                match user_remote.get_user(None).await {
                    Ok(user) => {
                        let mut guard = lock_w!(FDOLL);
                        guard.user_data.user = Some(user);
                    }
                    Err(error) => {
                        warn!("Failed to fetch user profile: {}", error);
                        show_refresh_error_dialog(
                            "Network Error",
                            "Failed to fetch user profile. You may be offline.",
                        );
                        return Err(());
                    }
                }
            }

            if matches!(scope, AppDataRefreshScope::All | AppDataRefreshScope::Friends) {
                match friend_remote.get_friends().await {
                    Ok(friends_list) => {
                        let mut guard = lock_w!(FDOLL);
                        guard.user_data.friends = Some(friends_list);
                        drop(guard);
                        friends::sync_from_app_data();
                    }
                    Err(error) => {
                        warn!("Failed to fetch friends list: {}", error);
                        show_refresh_error_dialog(
                            "Network Error",
                            "Failed to fetch friends list. You may be offline.",
                        );
                        return Err(());
                    }
                }
            }

            if matches!(scope, AppDataRefreshScope::All | AppDataRefreshScope::Dolls) {
                match dolls_remote.get_dolls().await {
                    Ok(dolls) => {
                        let mut guard = lock_w!(FDOLL);
                        guard.user_data.dolls = Some(dolls);
                    }
                    Err(error) => {
                        warn!("Failed to fetch dolls list: {}", error);
                        show_refresh_error_dialog(
                            "Network Error",
                            "Failed to fetch dolls list. You may be offline.",
                        );
                        return Err(());
                    }
                }
            }

            emit_refresh_events(scope);

            Ok(())
        }
        .await;

        {
            let mut in_flight = REFRESH_IN_FLIGHT.lock().await;
            in_flight.remove(&scope);
        }

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
    friends::clear();
}

fn emit_refresh_events(scope: AppDataRefreshScope) {
    let guard = lock_r!(FDOLL);
    let app_data = guard.user_data.clone();
    drop(guard);

    if let Err(error) = AppDataRefreshed(app_data).emit(get_app_handle()) {
        warn!("Failed to emit app-data-refreshed event: {}", error);
        show_refresh_error_dialog(
            "Sync Error",
            "Could not broadcast refreshed data to the UI. Some data may be stale.",
        );
    }

    if matches!(
        scope,
        AppDataRefreshScope::All | AppDataRefreshScope::User | AppDataRefreshScope::Dolls
    ) {
        match sprite::get_active_doll_sprite_base64() {
            Ok(sprite_b64) => {
                if let Err(error) = ActiveDollSpriteChanged(sprite_b64).emit(get_app_handle()) {
                    warn!("Failed to emit active-doll-sprite-changed event: {}", error);
                }
            }
            Err(error) => {
                warn!("Failed to generate active doll sprite: {}", error);
            }
        }
    }
}

fn show_refresh_error_dialog(title: &str, message: &str) {
    let handle = get_app_handle();
    MessageDialogBuilder::new(handle.dialog().clone(), title, message)
        .kind(MessageDialogKind::Error)
        .show(|_| {});
}
