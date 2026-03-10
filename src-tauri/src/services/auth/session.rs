use tracing::info;

use crate::get_app_handle;
use crate::init::lifecycle::construct_user_session;
use crate::services::scene::close_splash_window;
use crate::services::welcome::close_welcome_window;
use crate::state::auth::get_auth_pass_with_refresh;
use crate::{lock_w, state::FDOLL};

use super::storage::{clear_auth_pass, AuthError, AuthPass};

pub async fn get_session_token() -> Option<AuthPass> {
    get_auth_pass_with_refresh().await
}

pub async fn get_access_token() -> Option<String> {
    get_session_token().await.map(|pass| pass.access_token)
}

pub fn logout() -> Result<(), AuthError> {
    info!("Logging out user");
    lock_w!(FDOLL).auth.auth_pass = None;
    clear_auth_pass()?;
    Ok(())
}

pub async fn logout_and_restart() -> Result<(), AuthError> {
    logout()?;
    let app_handle = get_app_handle();
    app_handle.restart();
}

pub async fn login_and_init_session(email: &str, password: &str) -> Result<(), AuthError> {
    super::api::login(email, password).await?;
    close_welcome_window();
    tauri::async_runtime::spawn(async {
        construct_user_session().await;
        close_splash_window();
    });
    Ok(())
}
