use std::sync::{Arc, LazyLock, RwLock};

use tauri_specta::Event as _;
use tracing::warn;

use crate::{
    get_app_handle,
    models::app_state::{AppState, NekoPosition},
    services::app_events::AppStateChanged,
};

static APP_STATE: LazyLock<Arc<RwLock<AppState>>> =
    LazyLock::new(|| Arc::new(RwLock::new(AppState::default())));

pub fn get_snapshot() -> AppState {
    let guard = APP_STATE.read().expect("app state lock poisoned");
    guard.clone()
}

pub fn set_scene_setup_nekos_position(nekos_position: Option<NekoPosition>) {
    let mut guard = APP_STATE.write().expect("app state lock poisoned");
    guard.scene_setup.nekos_position = nekos_position;
    emit_snapshot(&guard);
}

pub fn set_scene_setup_nekos_opacity(nekos_opacity: f32) {
    let mut guard = APP_STATE.write().expect("app state lock poisoned");
    guard.scene_setup.nekos_opacity = nekos_opacity.clamp(0.1, 1.0);
    emit_snapshot(&guard);
}

pub fn set_scene_setup_nekos_scale(nekos_scale: f32) {
    let mut guard = APP_STATE.write().expect("app state lock poisoned");
    guard.scene_setup.nekos_scale = nekos_scale.clamp(0.5, 2.0);
    emit_snapshot(&guard);
}

fn emit_snapshot(app_state: &AppState) {
    if let Err(error) = AppStateChanged(app_state.clone()).emit(get_app_handle()) {
        warn!("Failed to emit app-state-changed event: {}", error);
    }
}
