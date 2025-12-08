use tracing::info;

use crate::{
    services::{
        auth::get_tokens, preferences::create_preferences_window, scene::create_scene_window,
    },
    state::init_app_data,
};

pub async fn start_fdoll() {
    bootstrap().await;
}

async fn construct_app() {
    init_app_data().await;
    create_scene_window();
    create_preferences_window();
}

pub async fn bootstrap() {
    match get_tokens().await {
        Some(_) => {
            info!("User session restored");
            construct_app().await;
        }
        None => {
            info!("No active session, user needs to authenticate");
            match crate::services::auth::init_auth_code_retrieval(|| {
                info!("Authentication successful, creating scene...");
                tauri::async_runtime::spawn(async {
                    info!("Creating scene after auth success...");
                    construct_app().await;
                });
            }) {
                Ok(it) => it,
                Err(err) => todo!("Handle authentication error: {}", err),
            };
        }
    }
}
