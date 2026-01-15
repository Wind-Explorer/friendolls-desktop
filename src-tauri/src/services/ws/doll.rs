use rust_socketio::{Payload, RawClient};
use tracing::{error, info};

use crate::{
    lock_r,
    state::{init_app_data_scoped, AppDataRefreshScope, FDOLL},
};

pub fn on_doll_created(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                info!("Received doll.created event: {:?}", first_value);

                // Refresh dolls list
                tauri::async_runtime::spawn(async {
                    init_app_data_scoped(AppDataRefreshScope::Dolls).await;
                });
            } else {
                info!("Received doll.created event with empty payload");
            }
        }
        _ => error!("Received unexpected payload format for doll.created"),
    }
}

pub fn on_doll_updated(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                info!("Received doll.updated event: {:?}", first_value);

                // Try to extract doll ID to check if it's the active doll
                let doll_id = first_value.get("id").and_then(|v| v.as_str());

                let is_active_doll = if let Some(id) = doll_id {
                    let guard = lock_r!(FDOLL);
                    guard
                        .app_data
                        .user
                        .as_ref()
                        .and_then(|u| u.active_doll_id.as_ref())
                        .map(|active_id| active_id == id)
                        .unwrap_or(false)
                } else {
                    false
                };

                // Refresh dolls + potentially User/Friends if active doll
                tauri::async_runtime::spawn(async move {
                    init_app_data_scoped(AppDataRefreshScope::Dolls).await;
                    if is_active_doll {
                        init_app_data_scoped(AppDataRefreshScope::User).await;
                        init_app_data_scoped(AppDataRefreshScope::Friends).await;
                    }
                });
            } else {
                info!("Received doll.updated event with empty payload");
            }
        }
        _ => error!("Received unexpected payload format for doll.updated"),
    }
}

pub fn on_doll_deleted(payload: Payload, _socket: RawClient) {
    match payload {
        Payload::Text(values) => {
            if let Some(first_value) = values.first() {
                info!("Received doll.deleted event: {:?}", first_value);

                // Try to extract doll ID to check if it was the active doll
                let doll_id = first_value.get("id").and_then(|v| v.as_str());

                let is_active_doll = if let Some(id) = doll_id {
                    let guard = lock_r!(FDOLL);
                    guard
                        .app_data
                        .user
                        .as_ref()
                        .and_then(|u| u.active_doll_id.as_ref())
                        .map(|active_id| active_id == id)
                        .unwrap_or(false)
                } else {
                    false
                };

                // Refresh dolls + User/Friends if the deleted doll was active
                tauri::async_runtime::spawn(async move {
                    init_app_data_scoped(AppDataRefreshScope::Dolls).await;
                    if is_active_doll {
                        init_app_data_scoped(AppDataRefreshScope::User).await;
                        init_app_data_scoped(AppDataRefreshScope::Friends).await;
                    }
                });
            } else {
                info!("Received doll.deleted event with empty payload");
            }
        }
        _ => error!("Received unexpected payload format for doll.deleted"),
    }
}
