use std::{collections::HashMap, sync::Mutex};

use tauri::{Manager, WebviewUrl, WebviewWindow, WindowEvent};
use tracing::{error, info};
use url::form_urlencoded;

use crate::{get_app_handle, utilities::toggle_macos_accessory_mode};

static WINDOW_KINDS: std::sync::OnceLock<Mutex<HashMap<String, WindowKind>>> =
    std::sync::OnceLock::new();

fn window_kinds() -> &'static Mutex<HashMap<String, WindowKind>> {
    WINDOW_KINDS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone, Copy)]
pub enum WindowKind {
    RegularUi,
    Accessory,
}

pub struct WindowConfig<'a> {
    pub label: &'a str,
    pub url_path: String,
    pub title: &'a str,
    pub width: f64,
    pub height: f64,
    pub resizable: bool,
    pub maximizable: bool,
    pub decorations: bool,
    pub transparent: bool,
    pub shadow: bool,
    pub visible: bool,
    pub skip_taskbar: bool,
    pub always_on_top: bool,
    pub visible_on_all_workspaces: bool,
    pub parent_label: Option<&'a str>,
    pub require_parent: bool,
    pub kind: WindowKind,
}

impl<'a> WindowConfig<'a> {
    pub fn regular_ui(label: &'a str, url_path: impl Into<String>, title: &'a str) -> Self {
        Self {
            label,
            url_path: url_path.into(),
            title,
            width: 420.0,
            height: 420.0,
            resizable: false,
            maximizable: false,
            decorations: true,
            transparent: false,
            shadow: true,
            visible: true,
            skip_taskbar: false,
            always_on_top: false,
            visible_on_all_workspaces: false,
            parent_label: None,
            require_parent: false,
            kind: WindowKind::RegularUi,
        }
    }

    pub fn accessory(label: &'a str, url_path: impl Into<String>, title: &'a str) -> Self {
        Self {
            label,
            url_path: url_path.into(),
            title,
            width: 600.0,
            height: 500.0,
            resizable: false,
            maximizable: false,
            decorations: false,
            transparent: true,
            shadow: false,
            visible: true,
            skip_taskbar: true,
            always_on_top: true,
            visible_on_all_workspaces: false,
            parent_label: None,
            require_parent: false,
            kind: WindowKind::Accessory,
        }
    }
}

pub enum EnsureWindowResult {
    Existing(WebviewWindow),
    Created(WebviewWindow),
}

pub enum EnsureWindowError {
    ShowExisting(tauri::Error),
    MissingParent(String),
    SetParent(tauri::Error),
    Build(tauri::Error),
}

fn apply_existing_window_behavior(
    window: &WebviewWindow,
    label: &str,
    show_existing: bool,
    focus_existing: bool,
) -> Result<(), EnsureWindowError> {
    if show_existing {
        if let Err(e) = window.show() {
            return Err(EnsureWindowError::ShowExisting(e));
        }
    }

    if focus_existing {
        if let Err(e) = window.set_focus() {
            error!("Failed to focus existing '{}' window: {}", label, e);
        }
    }

    Ok(())
}

pub fn encode_query_value(value: &str) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("v", value);
    let encoded = serializer.finish();
    encoded
        .strip_prefix("v=")
        .unwrap_or(encoded.as_str())
        .to_string()
}

pub fn ensure_window(
    config: &WindowConfig,
    show_existing: bool,
    focus_existing: bool,
) -> Result<EnsureWindowResult, EnsureWindowError> {
    let app_handle = get_app_handle();

    if let Some(window) = app_handle.get_webview_window(config.label) {
        apply_existing_window_behavior(&window, config.label, show_existing, focus_existing)?;

        if let Ok(mut guard) = window_kinds().lock() {
            guard.insert(config.label.to_string(), config.kind);
        }

        sync_macos_accessory_mode_for_current_windows();
        return Ok(EnsureWindowResult::Existing(window));
    }

    let mut builder = tauri::WebviewWindowBuilder::new(
        app_handle,
        config.label,
        WebviewUrl::App(config.url_path.clone().into()),
    )
    .title(config.title)
    .inner_size(config.width, config.height)
    .resizable(config.resizable)
    .decorations(config.decorations)
    .transparent(config.transparent)
    .shadow(config.shadow)
    .visible(config.visible)
    .skip_taskbar(config.skip_taskbar)
    .always_on_top(config.always_on_top)
    .visible_on_all_workspaces(config.visible_on_all_workspaces);

    builder = builder.maximizable(config.maximizable);

    if let Some(parent_label) = config.parent_label {
        if let Some(parent) = app_handle.get_webview_window(parent_label) {
            builder = builder
                .parent(&parent)
                .map_err(EnsureWindowError::SetParent)?;
        } else if config.require_parent {
            return Err(EnsureWindowError::MissingParent(parent_label.to_string()));
        }
    }

    match builder.build() {
        Ok(window) => {
            info!("{} window builder succeeded", config.label);
            if let Ok(mut guard) = window_kinds().lock() {
                guard.insert(config.label.to_string(), config.kind);
            }
            attach_macos_accessory_mode_listener(&window);
            sync_macos_accessory_mode_for_current_windows();
            Ok(EnsureWindowResult::Created(window))
        }
        Err(e) => {
            if let Some(window) = app_handle.get_webview_window(config.label) {
                apply_existing_window_behavior(
                    &window,
                    config.label,
                    show_existing,
                    focus_existing,
                )?;

                if let Ok(mut guard) = window_kinds().lock() {
                    guard.insert(config.label.to_string(), config.kind);
                }

                sync_macos_accessory_mode_for_current_windows();
                return Ok(EnsureWindowResult::Existing(window));
            }

            Err(EnsureWindowError::Build(e))
        }
    }
}

pub fn attach_macos_accessory_mode_listener(window: &WebviewWindow) {
    let label = window.label().to_string();
    window.on_window_event(move |event| {
        if let WindowEvent::Destroyed = event {
            if let Ok(mut guard) = window_kinds().lock() {
                guard.remove(&label);
            }
            info!(
                "Window '{}' destroyed, syncing macOS activation policy",
                label
            );
            sync_macos_accessory_mode_for_current_windows();
        }
    });
}

pub fn sync_macos_accessory_mode_for_current_windows() {
    #[cfg(target_os = "macos")]
    {
        let app_handle = get_app_handle();
        let has_regular_ui_window = if let Ok(guard) = window_kinds().lock() {
            guard.iter().any(|(label, kind)| {
                if !matches!(kind, WindowKind::RegularUi) {
                    return false;
                }

                if let Some(window) = app_handle.get_webview_window(label.as_str()) {
                    return window.is_visible().unwrap_or(false);
                }

                false
            })
        } else {
            false
        };

        info!(
            "Syncing macOS accessory mode: has_regular_ui_window={}, policy={}",
            has_regular_ui_window,
            if has_regular_ui_window {
                "Regular"
            } else {
                "Accessory"
            }
        );
        toggle_macos_accessory_mode(!has_regular_ui_window);
    }
}
