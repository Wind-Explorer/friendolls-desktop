pub mod app;
pub mod app_state;
pub mod auth;
pub mod config;
pub mod dolls;
pub mod friends;
pub mod interaction;
pub mod petpet;
pub mod sprite;

use crate::lock_r;
use crate::{
    services::app_data::{init_app_data_scoped, AppDataRefreshScope},
    state::FDOLL,
};
use tauri::async_runtime;

/// Helper to execute a mutation operation and refresh app data scopes in the background.
/// Returns the result of the operation after mapping errors to strings.
///
/// # Example
/// ```ignore
/// pub async fn create_doll(dto: CreateDollDto) -> Result<DollDto, String> {
///     let result = DollsRemote::new().create_doll(dto).await.map_err(|e| e.to_string())?;
///     refresh_app_data(&[AppDataRefreshScope::Dolls]).await;
///     Ok(result)
/// }
/// ```
pub async fn refresh_app_data(scopes: &[AppDataRefreshScope]) {
    let scopes = scopes.to_vec();
    async_runtime::spawn(async move {
        for scope in scopes {
            init_app_data_scoped(scope).await;
        }
    });
}

/// Helper to execute a mutation operation with conditional refresh.
///
/// # Example
/// ```ignore
/// pub async fn delete_doll(id: String) -> Result<(), String> {
///     let result = DollsRemote::new().delete_doll(&id).await.map_err(|e| e.to_string())?;
///     let is_active = is_active_doll(&id);
///     refresh_app_data_conditionally(&[AppDataRefreshScope::Dolls],
///                                   is_active.then_some(&[AppDataRefreshScope::User, AppDataRefreshScope::Friends]));
///     Ok(result)
/// }
/// ```
pub async fn refresh_app_data_conditionally(
    base_scopes: &[AppDataRefreshScope],
    conditional_scopes: Option<&[AppDataRefreshScope]>,
) {
    let mut all_scopes = base_scopes.to_vec();
    if let Some(extra_scopes) = conditional_scopes {
        all_scopes.extend_from_slice(extra_scopes);
    }
    refresh_app_data(&all_scopes).await;
}

/// Helper to check if a doll is currently the active doll.
/// Used in doll mutation operations to determine if additional refreshes are needed.
pub fn is_active_doll(doll_id: &str) -> bool {
    let guard = lock_r!(FDOLL);
    guard
        .user_data
        .user
        .as_ref()
        .and_then(|u| u.active_doll_id.as_ref())
        .map(|active_id| active_id == doll_id)
        .unwrap_or(false)
}
