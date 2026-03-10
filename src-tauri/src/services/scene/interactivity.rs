use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use device_query::{DeviceQuery, DeviceState, Keycode};
use once_cell::sync::OnceCell;
use tauri::Manager;
use tauri_specta::Event as _;
use tracing::{error, info, warn};

use crate::{get_app_handle, services::app_events::SceneInteractiveChanged};

use super::windows::SCENE_WINDOW_LABEL;

static SCENE_INTERACTIVE_STATE: OnceCell<Arc<AtomicBool>> = OnceCell::new();
static MODIFIER_LISTENER_INIT: OnceCell<()> = OnceCell::new();
static OPEN_PET_MENUS: OnceCell<Arc<std::sync::Mutex<std::collections::HashSet<String>>>> =
    OnceCell::new();

fn get_open_pet_menus() -> Arc<std::sync::Mutex<std::collections::HashSet<String>>> {
    OPEN_PET_MENUS
        .get_or_init(|| Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())))
        .clone()
}

fn scene_interactive_state() -> Arc<AtomicBool> {
    SCENE_INTERACTIVE_STATE
        .get_or_init(|| Arc::new(AtomicBool::new(false)))
        .clone()
}

pub(crate) fn start_scene_modifier_listener() {
    MODIFIER_LISTENER_INIT.get_or_init(|| {
        let state = scene_interactive_state();
        update_scene_interactive(false, false);

        let app_handle = get_app_handle().clone();

        #[cfg(target_os = "macos")]
        unsafe {
            info!("Accessibility status: {}", AXIsProcessTrusted());
            if !AXIsProcessTrusted() {
                error!(
                    "Accessibility permissions not granted. Global modifier listener will NOT start."
                );

                use tauri_plugin_dialog::DialogExt;
                use tauri_plugin_dialog::MessageDialogBuilder;
                use tauri_plugin_dialog::MessageDialogKind;

                MessageDialogBuilder::new(
                    app_handle.dialog().clone(),
                    "Missing Permissions",
                    "Friendolls needs Accessibility permissions to detect the Alt key for interactivity. Please grant permissions in System Settings -> Privacy & Security -> Accessibility and restart the app.",
                )
                .kind(MessageDialogKind::Warning)
                .show(|_| {});

                return;
            }
        }

        thread::spawn(move || {
            let device_state = DeviceState::new();
            let mut last_interactive = false;

            loop {
                let keys = device_state.get_keys();
                let keys_interactive =
                    (keys.contains(&Keycode::LAlt) || keys.contains(&Keycode::RAlt))
                        || keys.contains(&Keycode::Command);

                let menus_open = {
                    if let Ok(menus) = get_open_pet_menus().lock() {
                        !menus.is_empty()
                    } else {
                        false
                    }
                };

                let interactive = keys_interactive || menus_open;

                if interactive != last_interactive {
                    info!("Interactive state changed to: {}", interactive);
                    let previous = state.swap(interactive, Ordering::SeqCst);
                    if previous != interactive {
                        update_scene_interactive(interactive, false);
                    }
                    last_interactive = interactive;
                }

                thread::sleep(std::time::Duration::from_millis(100));
            }
        });
    });
}

pub(crate) fn update_scene_interactive(interactive: bool, should_click: bool) {
    let app_handle = get_app_handle();
    scene_interactive_state().store(interactive, Ordering::SeqCst);

    if !interactive {
        if let Ok(mut menus) = get_open_pet_menus().lock() {
            menus.clear();
        }
    }

    if let Some(window) = app_handle.get_window(SCENE_WINDOW_LABEL) {
        if let Err(e) = window.set_ignore_cursor_events(!interactive) {
            error!("Failed to toggle scene cursor events: {}", e);
        }

        if should_click {
            if let Some(pos) = crate::services::cursor::get_latest_cursor_position() {
                use enigo::{Button, Direction, Enigo, Mouse, Settings};

                match Enigo::new(&Settings::default()) {
                    Ok(mut enigo) => {
                        let _ = enigo.button(Button::Left, Direction::Click);
                        info!("Simulated click at ({}, {})", pos.x, pos.y);
                    }
                    Err(e) => {
                        error!("Failed to initialize Enigo for clicking: {}", e);
                    }
                }
            } else {
                warn!("Cannot click: No cursor position available yet");
            }
        }

        if let Err(e) = SceneInteractiveChanged(interactive).emit(&window) {
            error!("Failed to emit scene interactive event: {}", e);
        }
    } else {
        warn!("Scene window not available for interactive update");
    }
}

#[tauri::command]
#[specta::specta]
pub fn set_scene_interactive(interactive: bool, should_click: bool) {
    update_scene_interactive(interactive, should_click);
}

#[tauri::command]
#[specta::specta]
pub fn get_scene_interactive() -> Result<bool, String> {
    Ok(scene_interactive_state().load(Ordering::SeqCst))
}

#[tauri::command]
#[specta::specta]
pub fn set_pet_menu_state(id: String, open: bool) {
    let menus_arc = get_open_pet_menus();
    let should_update = {
        if let Ok(mut menus) = menus_arc.lock() {
            if open {
                menus.insert(id);
                get_app_handle()
                    .get_window(SCENE_WINDOW_LABEL)
                    .expect("Scene window should be present")
                    .set_focus()
                    .expect("Scene window should be focused");
            } else {
                menus.remove(&id);
            }
            !menus.is_empty()
        } else {
            false
        }
    };

    if should_update {
        update_scene_interactive(true, false);
    }
}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}
