use tauri::async_runtime;

use crate::state::{init_app_data_scoped, AppDataRefreshScope};

/// Refresh app data with the given scope
pub fn refresh_app_data(scope: AppDataRefreshScope) {
    async_runtime::spawn(async move {
        init_app_data_scoped(scope).await;
    });
}

/// Refresh multiple scopes sequentially
#[allow(dead_code)]
pub fn refresh_app_data_multi(scopes: Vec<AppDataRefreshScope>) {
    async_runtime::spawn(async move {
        for scope in scopes {
            init_app_data_scoped(scope).await;
        }
    });
}

/// Refresh dolls and optionally user/friends if doll was active
pub fn refresh_with_active_doll_check(doll_id: Option<&str>) {
    let is_active = doll_id.map(super::utils::is_active_doll).unwrap_or(false);

    async_runtime::spawn(async move {
        init_app_data_scoped(AppDataRefreshScope::Dolls).await;
        if is_active {
            init_app_data_scoped(AppDataRefreshScope::User).await;
            init_app_data_scoped(AppDataRefreshScope::Friends).await;
        }
    });
}
