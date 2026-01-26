use std::fmt::Debug;
use ts_rs::TS;

use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tracing::error;

use crate::get_app_handle;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppMetadata {
    pub localized: Option<String>,
    pub unlocalized: Option<String>,
    pub app_icon_b64: Option<String>,
}

pub fn listen_for_active_app_changes<F>(callback: F)
where
    F: Fn(AppMetadata) + Send + 'static,
{
    listen_impl(callback)
}

#[cfg(target_os = "macos")]
fn listen_impl<F>(callback: F)
where
    F: Fn(AppMetadata) + Send + 'static,
{
    macos::listen_for_active_app_changes(callback);
}

#[cfg(target_os = "windows")]
fn listen_impl<F>(callback: F)
where
    F: Fn(AppMetadata) + Send + 'static,
{
    windows_impl::listen_for_active_app_changes(callback);
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn listen_impl<F>(_callback: F)
where
    F: Fn(AppMetadata) + Send + 'static,
{
    // no-op on unsupported platforms
}

#[cfg(target_os = "macos")]
mod macos {
    use super::AppMetadata;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use lazy_static::lazy_static;
    use objc2::runtime::{AnyClass, AnyObject, Sel};
    use objc2::{class, msg_send, sel};
    use objc2_foundation::NSString;
    use std::ffi::CStr;
    use std::sync::{Mutex, Once};
    use tracing::info;

    #[allow(unused_imports)] // for framework linking, not referenced in code
    use objc2_app_kit::NSWorkspace;

    lazy_static! {
        static ref CALLBACK: Mutex<Option<Box<dyn Fn(AppMetadata) + Send + 'static>>> =
            Mutex::new(None);
    }
    static INIT_OBSERVER: Once = Once::new();

    pub fn listen_for_active_app_changes<F>(callback: F)
    where
        F: Fn(AppMetadata) + Send + 'static,
    {
        INIT_OBSERVER.call_once(|| {
            register_objc_observer_class();
        });
        *CALLBACK.lock().unwrap() = Some(Box::new(callback));

        unsafe {
            let cls = AnyClass::get(CStr::from_bytes_with_nul(b"RustActiveAppObserver\0").unwrap())
                .unwrap();
            let observer: *mut AnyObject = msg_send![cls, new];

            let ws: *mut AnyObject = msg_send![class!(NSWorkspace), sharedWorkspace];
            let nc: *mut AnyObject = msg_send![ws, notificationCenter];
            let notif_name = NSString::from_str("NSWorkspaceDidActivateApplicationNotification");
            let _: () = msg_send![
                nc,
                addObserver: observer,
                selector: sel!(appActivated:),
                name: &*notif_name,
                object: ws
            ];
        }
    }

    fn register_objc_observer_class() {
        use objc2::runtime::ClassBuilder;

        let cname = CStr::from_bytes_with_nul(b"RustActiveAppObserver\0").unwrap();
        let super_cls = class!(NSObject);
        let mut builder = ClassBuilder::new(&cname, super_cls).unwrap();
        unsafe {
            builder.add_method(
                sel!(appActivated:),
                app_activated as extern "C" fn(*mut AnyObject, Sel, *mut AnyObject),
            );
        }
        builder.register();
    }

    extern "C" fn app_activated(_self: *mut AnyObject, _cmd: Sel, _notif: *mut AnyObject) {
        let info = unsafe { get_active_app_metadata_macos() };
        if let Some(cb) = CALLBACK.lock().unwrap().as_ref() {
            cb(info);
        }
    }

    const ICON_SIZE: f64 = 64.0;
    const ICON_CACHE_LIMIT: usize = 50;

    lazy_static! {
        static ref ICON_CACHE: Mutex<std::collections::VecDeque<(String, String)>> =
            Mutex::new(std::collections::VecDeque::new());
    }

    fn get_cached_icon(path: &str) -> Option<String> {
        let mut cache = ICON_CACHE.lock().ok()?;
        if let Some(pos) = cache.iter().position(|(p, _)| p == path) {
            let (_, value) = cache.remove(pos)?;
            cache.push_front((path.to_string(), value.clone()));
            return Some(value);
        }
        None
    }

    fn put_cached_icon(path: &str, value: String) {
        if let Ok(mut cache) = ICON_CACHE.lock() {
            if let Some(pos) = cache.iter().position(|(p, _)| p == path) {
                cache.remove(pos);
            }
            cache.push_front((path.to_string(), value));
            if cache.len() > ICON_CACHE_LIMIT {
                cache.pop_back();
            }
        }
    }

    pub unsafe fn get_active_app_metadata_macos() -> AppMetadata {
        let ws: *mut AnyObject = msg_send![class!(NSWorkspace), sharedWorkspace];
        let front_app: *mut AnyObject = msg_send![ws, frontmostApplication];
        if front_app.is_null() {
            return AppMetadata {
                localized: None,
                unlocalized: None,
                app_icon_b64: None,
            };
        }

        let name: *mut NSString = msg_send![front_app, localizedName];
        let localized = if name.is_null() {
            None
        } else {
            Some(unsafe {
                CStr::from_ptr((*name).UTF8String())
                    .to_string_lossy()
                    .into_owned()
            })
        };

        let exe_url: *mut AnyObject = msg_send![front_app, executableURL];
        let unlocalized = if exe_url.is_null() {
            None
        } else {
            let exe_name: *mut NSString = msg_send![exe_url, lastPathComponent];
            if exe_name.is_null() {
                None
            } else {
                Some(unsafe {
                    CStr::from_ptr((*exe_name).UTF8String())
                        .to_string_lossy()
                        .into_owned()
                })
            }
        };

        let app_icon_b64 = get_active_app_icon_b64(front_app);

        AppMetadata {
            localized,
            unlocalized,
            app_icon_b64,
        }
    }

    fn get_active_app_icon_b64(front_app: *mut AnyObject) -> Option<String> {
        use std::slice;
        use tracing::warn;

        unsafe {
            let bundle_url: *mut AnyObject = msg_send![front_app, bundleURL];
            if bundle_url.is_null() {
                warn!("Failed to fetch icon: bundleURL null");
                return None;
            }

            let path: *mut NSString = msg_send![bundle_url, path];
            if path.is_null() {
                warn!("Failed to fetch icon: path null");
                return None;
            }

            let path_str = CStr::from_ptr((*path).UTF8String())
                .to_string_lossy()
                .into_owned();

            if let Some(cached) = get_cached_icon(&path_str) {
                return Some(cached);
            }

            let ws: *mut AnyObject = msg_send![class!(NSWorkspace), sharedWorkspace];
            let icon: *mut AnyObject = msg_send![ws, iconForFile: path];
            if icon.is_null() {
                warn!(
                    "Failed to fetch icon for {}: iconForFile returned null",
                    path_str
                );
                return None;
            }

            let _: () =
                msg_send![icon, setSize: objc2_foundation::NSSize::new(ICON_SIZE, ICON_SIZE)];

            let tiff: *mut AnyObject = msg_send![icon, TIFFRepresentation];
            if tiff.is_null() {
                warn!(
                    "Failed to fetch icon for {}: TIFFRepresentation null",
                    path_str
                );
                return None;
            }

            let rep: *mut AnyObject = msg_send![class!(NSBitmapImageRep), imageRepWithData: tiff];
            if rep.is_null() {
                warn!(
                    "Failed to fetch icon for {}: imageRepWithData null",
                    path_str
                );
                return None;
            }

            // 4 = NSBitmapImageFileTypePNG
            let png_data: *mut AnyObject = msg_send![
                rep,
                representationUsingType: 4u64,
                properties: std::ptr::null::<AnyObject>()
            ];
            if png_data.is_null() {
                warn!("Failed to fetch icon for {}: PNG data null", path_str);
                return None;
            }

            let bytes: *const u8 = msg_send![png_data, bytes];
            let len: usize = msg_send![png_data, length];
            if bytes.is_null() || len == 0 {
                warn!("Failed to fetch icon for {}: empty PNG data", path_str);
                return None;
            }

            let slice = slice::from_raw_parts(bytes, len);
            let encoded = STANDARD.encode(slice);
            put_cached_icon(&path_str, encoded.clone());

            Some(encoded)
        }
    }
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::AppMetadata;
    use std::ffi::OsString;
    use std::iter;
    use std::os::windows::ffi::OsStringExt;
    use std::path::Path;
    use std::ptr;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Globalization::GetUserDefaultLangID;
    use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowTextW,
        GetWindowThreadProcessId, EVENT_SYSTEM_FOREGROUND, MSG, WINEVENT_OUTOFCONTEXT,
    };

    pub fn listen_for_active_app_changes<F>(callback: F)
    where
        F: Fn(AppMetadata) + Send + 'static,
    {
        // Run the hook on a background thread so we don’t block the caller.
        std::thread::spawn(move || unsafe {
            let hook = SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                None,
                Some(win_event_proc::<F>),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            );

            if hook.is_invalid() {
                eprintln!("Failed to set event hook");
                return;
            }

            // store callback in TLS for this thread
            CALLBACK.with(|cb| cb.replace(Some(Box::new(callback))));

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                DispatchMessageW(&msg);
            }

            let _ = UnhookWinEvent(hook);
        });
    }

    thread_local! {
        static CALLBACK: std::cell::RefCell<Option<Box<dyn Fn(AppMetadata) + Send + 'static>>> =
            std::cell::RefCell::new(None);
    }

    unsafe extern "system" fn win_event_proc<F>(
        _h_win_event_hook: HWINEVENTHOOK,
        event: u32,
        hwnd: HWND,
        _id_object: i32,
        _id_child: i32,
        _id_event_thread: u32,
        _dw_ms_event_time: u32,
    ) {
        if event == EVENT_SYSTEM_FOREGROUND {
            let names = get_active_app_metadata_windows(Some(hwnd));
            CALLBACK.with(|cb| {
                if let Some(ref cb) = *cb.borrow() {
                    cb(names);
                }
            });
        }
    }

    pub fn get_active_app_metadata_windows(hwnd_override: Option<HWND>) -> AppMetadata {
        unsafe {
            let hwnd = hwnd_override.unwrap_or_else(|| GetForegroundWindow());
            if hwnd.0 == std::ptr::null_mut() {
                return AppMetadata {
                    localized: None,
                    unlocalized: None,
                    app_icon_b64: None,
                };
            }
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == 0 {
                return AppMetadata {
                    localized: None,
                    unlocalized: None,
                    app_icon_b64: None,
                };
            }
            let process_handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                Ok(h) if !h.is_invalid() => h,
                _ => {
                    return AppMetadata {
                        localized: None,
                        unlocalized: None,
                        app_icon_b64: None,
                    };
                }
            };

            let mut buffer: [u16; 1024] = [0; 1024];
            let size = GetModuleFileNameExW(process_handle, None, &mut buffer);
            let exe_path = if size > 0 {
                OsString::from_wide(&buffer[..size as usize])
            } else {
                return AppMetadata {
                    localized: None,
                    unlocalized: None,
                    app_icon_b64: None,
                };
            };
            let exe_path_str = exe_path.to_string_lossy();
            let exe_name = Path::new(&*exe_path_str)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let is_uwp = exe_name.eq_ignore_ascii_case("ApplicationFrameHost.exe");
            let (localized, unlocalized) = if is_uwp {
                (
                    get_window_title(hwnd),
                    Some("ApplicationFrameHost".to_string()),
                )
            } else {
                let unlocalized = Path::new(&*exe_path_str)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());
                let localized = get_file_description(&exe_path_str).or_else(|| unlocalized.clone());
                (localized, unlocalized)
            };
            AppMetadata {
                localized,
                unlocalized,
                app_icon_b64: None, // TODO: Implement icon fetching for Windows
            }
        }
    }

    fn get_window_title(hwnd: HWND) -> Option<String> {
        unsafe {
            let mut buffer: [u16; 512] = [0; 512];
            let len = GetWindowTextW(hwnd, &mut buffer);
            if len == 0 {
                None
            } else {
                Some(
                    OsString::from_wide(&buffer[..len as usize])
                        .to_string_lossy()
                        .into_owned(),
                )
            }
        }
    }

    fn get_file_description(exe_path: &str) -> Option<String> {
        unsafe {
            let wide_path: Vec<u16> = exe_path.encode_utf16().chain(iter::once(0)).collect();
            let size = windows::Win32::Storage::FileSystem::GetFileVersionInfoSizeW(
                PCWSTR(wide_path.as_ptr()),
                None,
            );
            if size == 0 {
                return None;
            }
            let mut buffer: Vec<u8> = vec![0; size as usize];
            let result = windows::Win32::Storage::FileSystem::GetFileVersionInfoW(
                PCWSTR(wide_path.as_ptr()),
                0,
                size,
                buffer.as_mut_ptr() as *mut _,
            );
            if result.is_err() {
                return None;
            }
            let mut translations_ptr: *mut u8 = ptr::null_mut();
            let mut translations_len: u32 = 0;
            let translation_query: Vec<u16> = "\\VarFileInfo\\Translation"
                .encode_utf16()
                .chain(iter::once(0))
                .collect();
            let trans_result = windows::Win32::Storage::FileSystem::VerQueryValueW(
                buffer.as_ptr() as *const _,
                PCWSTR(translation_query.as_ptr()),
                &mut translations_ptr as *mut _ as *mut *mut _,
                &mut translations_len,
            );
            let system_lang_id = GetUserDefaultLangID();
            if trans_result.as_bool() && translations_len >= 4 {
                let translations = std::slice::from_raw_parts(
                    translations_ptr as *const u16,
                    (translations_len / 2) as usize,
                );
                let mut lang_attempts = Vec::new();
                for i in (0..translations.len()).step_by(2) {
                    if i + 1 < translations.len() {
                        let lang = translations[i];
                        let codepage = translations[i + 1];
                        if lang == system_lang_id {
                            lang_attempts.insert(0, (lang, codepage));
                        } else {
                            lang_attempts.push((lang, codepage));
                        }
                    }
                }
                for (lang, codepage) in lang_attempts {
                    let query = format!(
                        "\\StringFileInfo\\{:04x}{:04x}\\FileDescription",
                        lang, codepage
                    );
                    let wide_query: Vec<u16> = query.encode_utf16().chain(iter::once(0)).collect();
                    let mut value_ptr: *mut u8 = ptr::null_mut();
                    let mut value_len: u32 = 0;
                    let query_result = windows::Win32::Storage::FileSystem::VerQueryValueW(
                        buffer.as_ptr() as *const _,
                        PCWSTR(wide_query.as_ptr()),
                        &mut value_ptr as *mut _ as *mut *mut _,
                        &mut value_len,
                    );
                    if query_result.as_bool() && !value_ptr.is_null() && value_len > 0 {
                        let wide_str = std::slice::from_raw_parts(
                            value_ptr as *const u16,
                            (value_len as usize).saturating_sub(1),
                        );
                        let description =
                            OsString::from_wide(wide_str).to_string_lossy().into_owned();
                        if !description.is_empty() {
                            return Some(description);
                        }
                    }
                }
            }
            let fallback_query: Vec<u16> = "\\StringFileInfo\\040904b0\\FileDescription"
                .encode_utf16()
                .chain(iter::once(0))
                .collect();
            let mut value_ptr: *mut u8 = ptr::null_mut();
            let mut value_len: u32 = 0;
            let query_result = windows::Win32::Storage::FileSystem::VerQueryValueW(
                buffer.as_ptr() as *const _,
                PCWSTR(fallback_query.as_ptr()),
                &mut value_ptr as *mut _ as *mut *mut _,
                &mut value_len,
            );
            if query_result.as_bool() && !value_ptr.is_null() && value_len > 0 {
                let wide_str = std::slice::from_raw_parts(
                    value_ptr as *const u16,
                    (value_len as usize).saturating_sub(1),
                );
                Some(OsString::from_wide(wide_str).to_string_lossy().into_owned())
            } else {
                None
            }
        }
    }
}

pub static ACTIVE_APP_CHANGED: &str = "active-app-changed";

pub fn init_active_app_changes_listener() {
    let app_handle = get_app_handle();
    listen_for_active_app_changes(|app_names: AppMetadata| {
        if let Err(e) = app_handle.emit(ACTIVE_APP_CHANGED, app_names) {
            error!("Failed to emit active app changed event: {}", e);
        }
    });
}
