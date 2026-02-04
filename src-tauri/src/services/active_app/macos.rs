use super::types::AppMetadata;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use lazy_static::lazy_static;
use objc2::runtime::{AnyClass, AnyObject, Sel};
use objc2::{class, msg_send, sel};
use objc2_foundation::NSString;
use std::ffi::CStr;
use std::sync::{Mutex, Once};
use tracing::warn;

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
        let cls =
            AnyClass::get(CStr::from_bytes_with_nul(b"RustActiveAppObserver\0").unwrap()).unwrap();
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
    let info = get_active_app_metadata_macos();
    if let Some(cb) = CALLBACK.lock().unwrap().as_ref() {
        cb(info);
    }
}

const ICON_SIZE: f64 = super::types::ICON_SIZE as f64;

fn get_cached_icon(path: &str) -> Option<String> {
    super::icon_cache::get(path)
}

fn put_cached_icon(path: &str, value: String) {
    super::icon_cache::put(path, value);
}

/// Retrieves metadata for the currently active application on macOS, including names and icon.
pub fn get_active_app_metadata_macos() -> AppMetadata {
    let ws: *mut AnyObject = unsafe { msg_send![class!(NSWorkspace), sharedWorkspace] };
    let front_app: *mut AnyObject = unsafe { msg_send![ws, frontmostApplication] };
    if front_app.is_null() {
        warn!("No frontmost application found");
        return AppMetadata {
            localized: None,
            unlocalized: None,
            app_icon_b64: None,
        };
    }

    let name: *mut NSString = unsafe { msg_send![front_app, localizedName] };
    let localized = if name.is_null() {
        warn!("Localized name is null for frontmost application");
        None
    } else {
        Some(unsafe {
            CStr::from_ptr((*name).UTF8String())
                .to_string_lossy()
                .into_owned()
        })
    };

    let exe_url: *mut AnyObject = unsafe { msg_send![front_app, executableURL] };
    let unlocalized = if exe_url.is_null() {
        warn!("Executable URL is null for frontmost application");
        None
    } else {
        let exe_name: *mut NSString = unsafe { msg_send![exe_url, lastPathComponent] };
        if exe_name.is_null() {
            warn!("Executable name is null");
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

        let _: () = msg_send![icon, setSize: objc2_foundation::NSSize::new(ICON_SIZE, ICON_SIZE)];

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
