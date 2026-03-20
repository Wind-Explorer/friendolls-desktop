use crate::{
    init::{lifecycle::validate_server_health, tracing::init_logging},
    services::{
        app_update::update_app,
        auth::get_session_token,
        cursor::init_cursor_tracking,
        presence_modules::init_modules,
        scene::{close_splash_window, open_splash_window},
        session::{construct_user_session, handle_disastrous_failure},
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
    update_app().await;
    init_app_state();
    init_system_tray();
    init_cursor_tracking().await;
    init_modules();

    if let Err(err) = validate_server_health().await {
        handle_disastrous_failure(Some(err.to_string())).await;
        return;
    }

    match get_session_token().await {
        Some(_tokens) => construct_user_session().await,
        None => open_welcome_window(),
    }

    close_splash_window();
}
