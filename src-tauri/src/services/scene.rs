use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use device_query::{DeviceQuery, DeviceState, Keycode};
use once_cell::sync::OnceCell;
use tauri::{Emitter, Manager};
use tauri_plugin_positioner::WindowExt;
use tracing::{error, info, warn};

use crate::{get_app_handle, services::app_events::AppEvents};

pub static SCENE_WINDOW_LABEL: &str = "scene";
pub static SPLASH_WINDOW_LABEL: &str = "splash";

static SCENE_INTERACTIVE_STATE: OnceCell<Arc<AtomicBool>> = OnceCell::new();
static MODIFIER_LISTENER_INIT: OnceCell<()> = OnceCell::new();

// New: Track which pets have open menus
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

pub fn update_scene_interactive(interactive: bool, should_click: bool) {
    let app_handle = get_app_handle();

    // If we are forcing interactive to false (e.g. background click), clear any open menus
    // This prevents the loop from immediately re-enabling it if the frontend hasn't updated yet
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
            // Get cursor position on screen and use Enigo to click
            if let Some(pos) = crate::services::cursor::get_latest_cursor_position() {
                use enigo::{Button, Direction, Enigo, Mouse, Settings};
                // Initialize Enigo with default settings, handling potential failure
                match Enigo::new(&Settings::default()) {
                    Ok(mut enigo) => {
                        // Perform a click (Press and Release)
                        // We ignore the result of the click action itself for now as it usually succeeds if init did
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

        if let Err(e) = window.emit(AppEvents::SceneInteractive.as_str(), &interactive) {
            error!("Failed to emit scene interactive event: {}", e);
        }
    } else {
        warn!("Scene window not available for interactive update");
    }
}

#[tauri::command]
pub fn set_scene_interactive(interactive: bool, should_click: bool) {
    update_scene_interactive(interactive, should_click);
}

#[tauri::command]
pub fn get_scene_interactive() -> Result<bool, String> {
    Ok(scene_interactive_state().load(Ordering::SeqCst))
}

#[tauri::command]
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

    // After updating state, re-evaluate interactivity immediately
    // We don't have direct access to key state here easily without recalculating everything,
    // but the loop will pick it up shortly.
    // HOWEVER, if we just closed the last menu and keys aren't held, we might want to ensure it closes fast.
    // For now, let the loop handle it to avoid race conditions with key states.
    // But if we just OPENED a menu, we definitely want to ensure interactive is TRUE.
    if should_update {
        update_scene_interactive(true, false);
    }
}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

fn start_scene_modifier_listener() {
    MODIFIER_LISTENER_INIT.get_or_init(|| {
        let state = scene_interactive_state();
        update_scene_interactive(false, false);

        let app_handle = get_app_handle().clone();

        #[cfg(target_os = "macos")]
        unsafe {
          info!("Accessibility status: {}", AXIsProcessTrusted());
            if !AXIsProcessTrusted() {
                // Warning only - polling might work without explicit permissions for just key state in some contexts,
                // or we just want to avoid the crash. We'll show the dialog but not return early if we want to try anyway.
                // However, usually global key monitoring requires it.
                // Let's show the dialog but NOT return, to try polling.
                // Or better, let's keep the return if we think it won't work at all,
                // but since the crash was the main issue, let's try to proceed safely.
                // For now, I will keep the dialog and the return to encourage users to enable it,
                // as it's likely needed for global input monitoring.

                // On second thought, let's keep the return to be safe and clear to the user.
                error!("Accessibility permissions not granted. Global modifier listener will NOT start.");

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

        // Spawn a thread for polling key state
        thread::spawn(move || {
            let device_state = DeviceState::new();
            let mut last_interactive = false;

            loop {
                let keys = device_state.get_keys();
                // Check for Alt key (Option on Mac)
                let keys_interactive = (keys.contains(&Keycode::LAlt) || keys.contains(&Keycode::RAlt)) || keys.contains(&Keycode::Command);

                // Check if any pet menus are open
                let menus_open = {
                    if let Ok(menus) = get_open_pet_menus().lock() {
                        !menus.is_empty()
                    } else {
                        false
                    }
                };

                let interactive = keys_interactive || menus_open;

                if interactive != last_interactive {
                    // State changed
                    info!("Interactive state changed to: {}", interactive);
                    let previous = state.swap(interactive, Ordering::SeqCst);
                    if previous != interactive {
                        update_scene_interactive(interactive, false);
                    }
                    last_interactive = interactive;
                }

                // Poll every 100ms
                thread::sleep(std::time::Duration::from_millis(100));
            }
        });
    });
}

pub fn overlay_fullscreen(window: &tauri::Window) -> Result<(), tauri::Error> {
    // Get the primary monitor
    let monitor = get_app_handle().primary_monitor()?.unwrap();
    let monitor_size = monitor.size();

    // Fullscreen the window by expanding the window to match monitor size then move it to the top-left corner
    // This forces the window to fit under the notch that exists on MacBooks with a notch
    window.set_size(tauri::PhysicalSize {
        width: monitor_size.width,
        height: monitor_size.height,
    })?;

    window.set_position(tauri::PhysicalPosition { x: 0, y: 0 })?;

    Ok(())
}

pub fn open_splash_window() {
    let app_handle = get_app_handle();
    let existing_webview_window = app_handle.get_window(SPLASH_WINDOW_LABEL);

    if let Some(window) = existing_webview_window {
        window.show().unwrap();
        return;
    }

    info!("Starting splash window creation...");
    let webview_window = match tauri::WebviewWindowBuilder::new(
        app_handle,
        SPLASH_WINDOW_LABEL,
        tauri::WebviewUrl::App("/splash".into()),
    )
    .title("Friendolls Splash")
    .inner_size(600.0, 300.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .visible(false) // Show it after centering
    .skip_taskbar(true)
    .always_on_top(true)
    .build()
    {
        Ok(window) => {
            info!("Splash window builder succeeded");
            window
        }
        Err(e) => {
            error!("Failed to build splash window: {}", e);
            return;
        }
    };

    if let Err(e) = webview_window.move_window(tauri_plugin_positioner::Position::Center) {
        error!("Failed to move splash window to center: {}", e);
        // Continue anyway
    }

    if let Err(e) = webview_window.show() {
        error!("Failed to show splash window: {}", e);
    }

    info!("Splash window initialized successfully.");
}

pub fn close_splash_window() {
    let app_handle = get_app_handle();
    if let Some(window) = app_handle.get_window(SPLASH_WINDOW_LABEL) {
        if let Err(e) = window.close() {
            error!("Failed to close splash window: {}", e);
        } else {
            info!("Splash window closed");
        }
    }
}

pub fn open_scene_window() {
    let app_handle = get_app_handle();
    let existing_webview_window = app_handle.get_window(SCENE_WINDOW_LABEL);

    if let Some(window) = existing_webview_window {
        window.show().unwrap();
        return;
    }

    info!("Starting scene creation...");
    let webview_window = match tauri::WebviewWindowBuilder::new(
        app_handle,
        SCENE_WINDOW_LABEL,
        tauri::WebviewUrl::App("/scene".into()),
    )
    .title("Friendolls Scene")
    .inner_size(600.0, 500.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .visible(true)
    .skip_taskbar(true)
    .always_on_top(true)
    .visible_on_all_workspaces(true)
    .build()
    {
        Ok(window) => {
            info!("Scene window builder succeeded");
            window
        }
        Err(e) => {
            error!("Failed to build scene window: {}", e);
            return;
        }
    };

    if let Err(e) = webview_window.move_window(tauri_plugin_positioner::Position::Center) {
        error!("Failed to move scene window to center: {}", e);
        return;
    }

    let window = match get_app_handle().get_window(webview_window.label()) {
        Some(window) => window,
        None => {
            error!("Failed to get scene window after creation");
            return;
        }
    };

    if let Err(e) = overlay_fullscreen(&window) {
        error!("Failed to set overlay fullscreen: {}", e);
        return;
    }

    if let Err(e) = window.set_ignore_cursor_events(true) {
        error!("Failed to set ignore cursor events: {}", e);
        return;
    }

    // Start global modifier listener once scene window exists
    start_scene_modifier_listener();

    #[cfg(debug_assertions)]
    webview_window.open_devtools();

    info!("Scene window initialized successfully.");
}
