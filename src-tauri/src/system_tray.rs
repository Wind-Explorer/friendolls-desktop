use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tracing::error;

use crate::{get_app_handle, services::app_menu::open_app_menu_window};

pub fn init_system_tray() {
    let app = get_app_handle();

    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
    let open_app_menu_i =
        MenuItem::with_id(app, "open-app-menu", "Open App Menu", true, None::<&str>).unwrap();

    let menu = match Menu::with_items(app, &[&open_app_menu_i, &quit_i]) {
        Ok(it) => it,
        Err(err) => todo!("Handle error: {}", err),
    };

    match TrayIconBuilder::new()
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
    {
        Ok(it) => it,
        Err(err) => todo!("Handle error: {}", err),
    };
}
