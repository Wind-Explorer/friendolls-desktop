#[cfg(target_os = "windows")]
use tauri::WebviewWindow;
use tauri::{Listener, Manager};
use tauri_specta::Event as _;
use tracing::{error, info};

use crate::{
    get_app_handle,
    services::app_events::{CreateDoll, EditDoll, SetInteractionOverlay},
};

static APP_MENU_WINDOW_LABEL: &str = "app_menu";

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{BOOL, HWND};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow; // Correct location for EnableWindow

// #[cfg(target_os = "macos")]
// use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};
// #[cfg(target_os = "macos")]
// use tauri::v2_compat::MsgSend; // Removed: Not needed/doesn't exist

/// Helper to disable/enable interaction with a window
#[cfg(target_os = "windows")]
fn set_window_interaction(window: &WebviewWindow, enable: bool) {
    {
        // Add explicit import for the trait method
        use raw_window_handle::HasWindowHandle;

        if let Ok(handle) = window.window_handle() {
            // raw-window-handle 0.6 uses a match pattern
            // The trait returns a WindowHandle wrapper which has as_raw()
            match handle.as_raw() {
                raw_window_handle::RawWindowHandle::Win32(win32_handle) => {
                    // win32_handle.hwnd is a NonZeroIsize (or similar depending on version), cast to isize then HWND
                    // windows crate expects HWND(isize) or HWND(*mut c_void) depending on version
                    // raw-window-handle 0.6: hwnd is NonZero<isize>
                    let hwnd_isize = win32_handle.hwnd.get();
                    let hwnd = HWND(hwnd_isize as _);

                    unsafe {
                        // TRUE (1) to enable, FALSE (0) to disable
                        let _ = EnableWindow(hwnd, BOOL::from(enable));
                    }
                }
                _ => {
                    error!("Unsupported window handle type on Windows");
                }
            }
        } else {
            error!("Failed to get window handle");
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn open_doll_editor_window(doll_id: Option<String>) {
    let app_handle = get_app_handle().clone();

    // Dispatch to main thread to avoid potential deadlocks on Windows when setting parent window
    let _ = app_handle.run_on_main_thread(move || {
        let app_handle = get_app_handle();

        // Construct a unique window label
        let window_label = if let Some(ref id) = doll_id {
            format!("doll_editor_{}", id)
        } else {
            "doll_editor_create".to_string()
        };

        // Check if the window already exists
        let existing_window = app_handle.get_webview_window(&window_label);
        if let Some(window) = existing_window {
            // If it exists, we might want to reload it with new params or just focus it
            if let Err(e) = window.set_focus() {
                error!("Failed to focus existing doll editor window: {}", e);
            }

            // Ensure overlay is active on parent (redundancy for safety)
            #[cfg(target_os = "macos")]
            if let Some(parent) = app_handle.get_webview_window(APP_MENU_WINDOW_LABEL) {
                if let Err(e) = SetInteractionOverlay(true).emit(&parent) {
                    error!("Failed to ensure interaction overlay on parent: {}", e);
                }
            }

            // Emit event to update context
            if let Some(id) = doll_id {
                if let Err(e) = EditDoll(id).emit(&window) {
                    error!("Failed to emit edit-doll event: {}", e);
                }
            } else if let Err(e) = CreateDoll.emit(&window) {
                error!("Failed to emit create-doll event: {}", e);
            }

            return;
        }

        let url_path = if let Some(id) = doll_id {
            format!("/doll-editor?id={}", id)
        } else {
            "/doll-editor".to_string()
        };

        let mut builder = tauri::WebviewWindowBuilder::new(
            app_handle,
            &window_label,
            tauri::WebviewUrl::App(url_path.into()),
        )
        .title("Doll Editor")
        .inner_size(300.0, 400.0)
        .resizable(false)
        .maximizable(false)
        .decorations(true)
        .transparent(false)
        .shadow(true)
        .visible(true)
        .skip_taskbar(false)
        .always_on_top(true) // Helper window, nice to stay on top
        .visible_on_all_workspaces(false);

        // Set parent if app menu exists
        // Also disable interaction with parent while child is open

        // macOS Specific: Focus Trap Listener ID
        // We need to capture this to unlisten later.

        let mut parent_focus_listener_id: Option<u32> = None;

        if let Some(parent) = app_handle.get_webview_window(APP_MENU_WINDOW_LABEL) {
            // 1. Disable parent interaction immediately (Windows only)
            #[cfg(target_os = "windows")]
            set_window_interaction(&parent, false);

            // 2. Setup Focus Trap (macOS only)
            #[cfg(target_os = "macos")]
            {
                let child_label = window_label.clone();
                let app_handle_clone = get_app_handle().clone();

                // Emit event to show overlay
                if let Err(e) = SetInteractionOverlay(true).emit(&parent) {
                    error!("Failed to emit set-interaction-overlay event: {}", e);
                }

                // Listen for when the PARENT gets focus
                let id = parent.listen("tauri://focus", move |_| {
                    info!(
                        "Parent focused, redirecting focus to child: {}",
                        child_label
                    );
                    if let Some(child) = app_handle_clone.get_webview_window(&child_label) {
                        if let Err(e) = child.set_focus() {
                            error!("Failed to refocus child window: {}", e);
                        }
                    }
                });
                parent_focus_listener_id = Some(id);
            }

            match builder.parent(&parent) {
                Ok(b) => builder = b,
                Err(e) => {
                    error!("Failed to set parent for doll editor window: {}", e);
                    // If we fail, revert changes
                    #[cfg(target_os = "windows")]
                    set_window_interaction(&parent, true);

                    #[cfg(target_os = "macos")]
                    {
                        if let Some(id) = parent_focus_listener_id {
                            parent.unlisten(id);
                        }
                        // Remove overlay if we failed
                        let _ = SetInteractionOverlay(false).emit(&parent);
                    }
                    return;
                }
            };
        }

        match builder.build() {
            Ok(window) => {
                info!("{} window builder succeeded", window_label);

                // 3. Setup cleanup hook: When this child window is destroyed, re-enable the parent
                let app_handle_clone = get_app_handle().clone();

                // Capture the listener ID for cleanup
                let listener_to_remove = parent_focus_listener_id;

                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Destroyed = event {
                        if let Some(parent) =
                            app_handle_clone.get_webview_window(APP_MENU_WINDOW_LABEL)
                        {
                            info!("Doll editor destroyed, restoring parent state");

                            // Windows: Re-enable input
                            #[cfg(target_os = "windows")]
                            set_window_interaction(&parent, true);

                            // macOS: Remove focus trap listener
                            #[cfg(target_os = "macos")]
                            {
                                if let Some(id) = listener_to_remove {
                                    parent.unlisten(id);
                                }
                                // Remove overlay
                                if let Err(e) = SetInteractionOverlay(false).emit(&parent) {
                                    error!("Failed to remove interaction overlay: {}", e);
                                }
                            }

                            // Optional: Focus parent after child closes for good UX
                            let _ = parent.set_focus();
                        }
                    }
                });

                // #[cfg(debug_assertions)]
                // window.open_devtools();
            }
            Err(e) => {
                error!("Failed to build {} window: {}", window_label, e);
                // If build failed, revert
                if let Some(parent) = get_app_handle().get_webview_window(APP_MENU_WINDOW_LABEL) {
                    #[cfg(target_os = "windows")]
                    set_window_interaction(&parent, true);

                    #[cfg(target_os = "macos")]
                    {
                        if let Some(id) = parent_focus_listener_id {
                            parent.unlisten(id);
                        }
                        let _ = SetInteractionOverlay(false).emit(&parent);
                    }
                }
            }
        };
    });
}
