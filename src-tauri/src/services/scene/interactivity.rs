use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use device_query::{DeviceQuery, DeviceState, Keycode};
use once_cell::sync::OnceCell;
use tauri::Manager;
use tauri_specta::Event as _;
use tracing::{error, info, warn};

use crate::{
    get_app_handle, lock_r,
    services::{
        app_events::SceneInteractiveChanged,
        client_config::{
            SceneInteractivityHotkey, SceneInteractivityKey, SceneInteractivityModifier,
        },
    },
    state::FDOLL,
};

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

fn has_modifier(keys: &[Keycode], modifier: SceneInteractivityModifier) -> bool {
    match modifier {
        SceneInteractivityModifier::Cmd => {
            keys.contains(&Keycode::Command)
                || keys.contains(&Keycode::RCommand)
                || keys.contains(&Keycode::LMeta)
                || keys.contains(&Keycode::RMeta)
        }
        SceneInteractivityModifier::Alt => {
            keys.contains(&Keycode::LAlt)
                || keys.contains(&Keycode::RAlt)
                || keys.contains(&Keycode::LOption)
                || keys.contains(&Keycode::ROption)
        }
        SceneInteractivityModifier::Ctrl => {
            keys.contains(&Keycode::LControl) || keys.contains(&Keycode::RControl)
        }
        SceneInteractivityModifier::Shift => {
            keys.contains(&Keycode::LShift) || keys.contains(&Keycode::RShift)
        }
    }
}

fn has_key(keys: &[Keycode], key: SceneInteractivityKey) -> bool {
    match key {
        SceneInteractivityKey::A => keys.contains(&Keycode::A),
        SceneInteractivityKey::B => keys.contains(&Keycode::B),
        SceneInteractivityKey::C => keys.contains(&Keycode::C),
        SceneInteractivityKey::D => keys.contains(&Keycode::D),
        SceneInteractivityKey::E => keys.contains(&Keycode::E),
        SceneInteractivityKey::F => keys.contains(&Keycode::F),
        SceneInteractivityKey::G => keys.contains(&Keycode::G),
        SceneInteractivityKey::H => keys.contains(&Keycode::H),
        SceneInteractivityKey::I => keys.contains(&Keycode::I),
        SceneInteractivityKey::J => keys.contains(&Keycode::J),
        SceneInteractivityKey::K => keys.contains(&Keycode::K),
        SceneInteractivityKey::L => keys.contains(&Keycode::L),
        SceneInteractivityKey::M => keys.contains(&Keycode::M),
        SceneInteractivityKey::N => keys.contains(&Keycode::N),
        SceneInteractivityKey::O => keys.contains(&Keycode::O),
        SceneInteractivityKey::P => keys.contains(&Keycode::P),
        SceneInteractivityKey::Q => keys.contains(&Keycode::Q),
        SceneInteractivityKey::R => keys.contains(&Keycode::R),
        SceneInteractivityKey::S => keys.contains(&Keycode::S),
        SceneInteractivityKey::T => keys.contains(&Keycode::T),
        SceneInteractivityKey::U => keys.contains(&Keycode::U),
        SceneInteractivityKey::V => keys.contains(&Keycode::V),
        SceneInteractivityKey::W => keys.contains(&Keycode::W),
        SceneInteractivityKey::X => keys.contains(&Keycode::X),
        SceneInteractivityKey::Y => keys.contains(&Keycode::Y),
        SceneInteractivityKey::Z => keys.contains(&Keycode::Z),
        SceneInteractivityKey::Num0 => keys.contains(&Keycode::Key0),
        SceneInteractivityKey::Num1 => keys.contains(&Keycode::Key1),
        SceneInteractivityKey::Num2 => keys.contains(&Keycode::Key2),
        SceneInteractivityKey::Num3 => keys.contains(&Keycode::Key3),
        SceneInteractivityKey::Num4 => keys.contains(&Keycode::Key4),
        SceneInteractivityKey::Num5 => keys.contains(&Keycode::Key5),
        SceneInteractivityKey::Num6 => keys.contains(&Keycode::Key6),
        SceneInteractivityKey::Num7 => keys.contains(&Keycode::Key7),
        SceneInteractivityKey::Num8 => keys.contains(&Keycode::Key8),
        SceneInteractivityKey::Num9 => keys.contains(&Keycode::Key9),
        SceneInteractivityKey::F1 => keys.contains(&Keycode::F1),
        SceneInteractivityKey::F2 => keys.contains(&Keycode::F2),
        SceneInteractivityKey::F3 => keys.contains(&Keycode::F3),
        SceneInteractivityKey::F4 => keys.contains(&Keycode::F4),
        SceneInteractivityKey::F5 => keys.contains(&Keycode::F5),
        SceneInteractivityKey::F6 => keys.contains(&Keycode::F6),
        SceneInteractivityKey::F7 => keys.contains(&Keycode::F7),
        SceneInteractivityKey::F8 => keys.contains(&Keycode::F8),
        SceneInteractivityKey::F9 => keys.contains(&Keycode::F9),
        SceneInteractivityKey::F10 => keys.contains(&Keycode::F10),
        SceneInteractivityKey::F11 => keys.contains(&Keycode::F11),
        SceneInteractivityKey::F12 => keys.contains(&Keycode::F12),
        SceneInteractivityKey::Enter => {
            keys.contains(&Keycode::Enter) || keys.contains(&Keycode::NumpadEnter)
        }
        SceneInteractivityKey::Space => keys.contains(&Keycode::Space),
        SceneInteractivityKey::Escape => keys.contains(&Keycode::Escape),
        SceneInteractivityKey::Tab => keys.contains(&Keycode::Tab),
        SceneInteractivityKey::Backspace => keys.contains(&Keycode::Backspace),
        SceneInteractivityKey::Delete => keys.contains(&Keycode::Delete),
        SceneInteractivityKey::Insert => keys.contains(&Keycode::Insert),
        SceneInteractivityKey::Home => keys.contains(&Keycode::Home),
        SceneInteractivityKey::End => keys.contains(&Keycode::End),
        SceneInteractivityKey::PageUp => keys.contains(&Keycode::PageUp),
        SceneInteractivityKey::PageDown => keys.contains(&Keycode::PageDown),
        SceneInteractivityKey::ArrowUp => keys.contains(&Keycode::Up),
        SceneInteractivityKey::ArrowDown => keys.contains(&Keycode::Down),
        SceneInteractivityKey::ArrowLeft => keys.contains(&Keycode::Left),
        SceneInteractivityKey::ArrowRight => keys.contains(&Keycode::Right),
        SceneInteractivityKey::Minus => keys.contains(&Keycode::Minus),
        SceneInteractivityKey::Equal => keys.contains(&Keycode::Equal),
        SceneInteractivityKey::LeftBracket => keys.contains(&Keycode::LeftBracket),
        SceneInteractivityKey::RightBracket => keys.contains(&Keycode::RightBracket),
        SceneInteractivityKey::BackSlash => keys.contains(&Keycode::BackSlash),
        SceneInteractivityKey::Semicolon => keys.contains(&Keycode::Semicolon),
        SceneInteractivityKey::Apostrophe => keys.contains(&Keycode::Apostrophe),
        SceneInteractivityKey::Comma => keys.contains(&Keycode::Comma),
        SceneInteractivityKey::Dot => keys.contains(&Keycode::Dot),
        SceneInteractivityKey::Slash => keys.contains(&Keycode::Slash),
        SceneInteractivityKey::Grave => keys.contains(&Keycode::Grave),
    }
}

fn pressed_modifiers(keys: &[Keycode]) -> Vec<SceneInteractivityModifier> {
    let mut modifiers = Vec::new();

    for modifier in [
        SceneInteractivityModifier::Cmd,
        SceneInteractivityModifier::Alt,
        SceneInteractivityModifier::Ctrl,
        SceneInteractivityModifier::Shift,
    ] {
        if has_modifier(keys, modifier) {
            modifiers.push(modifier);
        }
    }

    modifiers
}

fn is_hotkey_active(keys: &[Keycode], hotkey: SceneInteractivityHotkey) -> bool {
    if pressed_modifiers(keys) != hotkey.modifiers {
        return false;
    }

    hotkey.key.map_or(true, |key| has_key(keys, key))
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
                    "Friendolls needs Accessibility permissions to detect your scene interactivity hotkey. Please grant permissions in System Settings -> Privacy & Security -> Accessibility and restart the app.",
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
                let hotkey = {
                    let guard = lock_r!(FDOLL);
                    guard.app_config.scene_interactivity_hotkey.clone()
                };
                let keys_interactive = is_hotkey_active(&keys, hotkey);

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
