use crate::{
    get_app_handle, lock_r, lock_w, services::app_menu::open_app_menu_window, state::FDOLL,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tracing::error;

/// Constructs app system tray.
/// Uses Tauri.
pub fn init_system_tray() {
    let app = get_app_handle();

    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
    let open_app_menu_i =
        MenuItem::with_id(app, "open-app-menu", "Open App Menu", true, None::<&str>).unwrap();

    let menu = match Menu::with_items(app, &[&open_app_menu_i, &quit_i]) {
        Ok(it) => it,
        Err(err) => todo!("Handle error: {}", err),
    };

    let tray = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "open-app-menu" => {
                open_app_menu_window();
            }
            _ => {
                error!("menu item {:?} not handled", event.id);
            }
        })
        .icon(app.default_window_icon().unwrap().clone())
        .build(app)
        .unwrap_or_else(|err| panic!("Failed to build tray: {}", err));
    {
        let mut guard = lock_w!(FDOLL);
        guard.tray = Some(tray);
    };

    update_system_tray(false);
}

/// Toggle the "Open App Menu" item in the system tray.
/// Used for when user is signed in vs not signed in.
pub fn update_system_tray(is_logged_in: bool) {
    let app = get_app_handle();
    let guard = lock_r!(FDOLL);
    if let Some(tray) = &guard.tray {
        let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
        let menu = if is_logged_in {
            let open_app_menu_i =
                MenuItem::with_id(app, "open-app-menu", "Open App Menu", true, None::<&str>)
                    .unwrap();
            Menu::with_items(app, &[&open_app_menu_i, &quit_i])
        } else {
            Menu::with_items(app, &[&quit_i])
        };
        let menu = match menu {
            Ok(it) => it,
            Err(err) => {
                error!("Failed to create menu: {}", err);
                return;
            }
        };
        if let Err(err) = tray.set_menu(Some(menu)) {
            error!("Failed to update tray menu: {}", err);
        }
    }
}
