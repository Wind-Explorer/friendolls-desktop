use crate::{
    init::{
        lifecycle::{construct_user_session, handle_disasterous_failure, validate_server_health},
        tracing::init_logging,
    },
    services::{
        active_app::init_foreground_app_change_listener,
        auth::get_session_token,
        cursor::init_cursor_tracking,
        scene::{close_splash_window, open_splash_window},
        welcome::open_welcome_window,
    },
    state::init_app_state,
    system_tray::init_system_tray,
};

pub mod lifecycle;
pub mod tracing;

/// The very function that handles
/// init and startup of everything.
pub async fn launch_app() {
    init_logging();
    open_splash_window();
    init_app_state();
    init_system_tray();
    init_cursor_tracking().await;
    init_foreground_app_change_listener();

    if let Err(err) = validate_server_health().await {
        handle_disasterous_failure(Some(err.to_string())).await;
        return;
    }

    match get_session_token().await {
        Some(_tokens) => construct_user_session().await,
        None => open_welcome_window(),
    }

    close_splash_window();
}
