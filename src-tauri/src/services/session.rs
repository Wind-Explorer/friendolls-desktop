use crate::{
    services::{
        app_data::{clear_app_data, init_app_data_scoped, AppDataRefreshScope},
        health_manager::open_health_manager_window,
        health_monitor::{start_health_monitor, stop_health_monitor},
        scene::open_scene_window,
        session_windows::close_all_windows,
        ws::client::{clear_ws_client, establish_websocket_connection},
    },
    state::auth::{start_background_token_refresh, stop_background_token_refresh},
    system_tray::update_system_tray,
};

pub async fn construct_user_session() {
    connect_user_profile().await;
    close_all_windows();
    open_scene_window();
    update_system_tray(true);
}

pub async fn destruct_user_session() {
    disconnect_user_profile().await;
    close_all_windows();
    update_system_tray(false);
}

pub async fn handle_disastrous_failure(error_message: Option<String>) {
    destruct_user_session().await;
    open_health_manager_window(error_message);
}

async fn connect_user_profile() {
    init_app_data_scoped(AppDataRefreshScope::All).await;
    establish_websocket_connection().await;
    start_background_token_refresh().await;
    start_health_monitor().await;
}

async fn disconnect_user_profile() {
    stop_health_monitor();
    stop_background_token_refresh();
    clear_app_data();
    clear_ws_client().await;
}
