use tracing::info;
use tokio::time::{timeout, Duration};

use crate::get_app_handle;
use crate::services::{scene::close_splash_window, session::construct_user_session, welcome::close_welcome_window};
use crate::state::auth::get_auth_pass_with_refresh;
use crate::{lock_w, state::FDOLL};

use super::storage::{clear_auth_pass, AuthError, AuthPass};

pub async fn get_session_token() -> Option<AuthPass> {
    get_auth_pass_with_refresh().await
}

pub async fn get_access_token() -> Option<String> {
    get_session_token().await.map(|pass| pass.access_token)
}

pub async fn logout() -> Result<(), AuthError> {
    info!("Logging out user");
    let refresh_token = lock_w!(FDOLL)
        .auth
        .auth_pass
        .take()
        .and_then(|pass| pass.refresh_token);
    clear_auth_pass()?;

    if let Some(refresh_token) = refresh_token {
        match timeout(Duration::from_secs(5), super::api::logout_remote(&refresh_token)).await {
            Ok(Ok(())) => {}
            Ok(Err(err)) => info!("Failed to revoke refresh token on server: {}", err),
            Err(_) => info!("Timed out while revoking refresh token on server"),
        }
    }

    Ok(())
}

pub async fn logout_and_restart() -> Result<(), AuthError> {
    logout().await?;
    let app_handle = get_app_handle();
    app_handle.restart();
}

pub async fn finish_login_session() -> Result<(), AuthError> {
    close_welcome_window();
    tauri::async_runtime::spawn(async {
        construct_user_session().await;
        close_splash_window();
    });
    Ok(())
}

pub async fn start_browser_login(provider: &str) -> Result<(), AuthError> {
    super::flow::start_browser_auth_flow(provider).await
}
