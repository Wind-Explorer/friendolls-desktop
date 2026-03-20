#[macro_export]
macro_rules! lock_r {
    ($rwlock:expr) => {{
        match $rwlock.read() {
            Ok(guard) => guard,
            Err(_) => panic!(
                "Failed to acquire read lock on {} at {}:{}",
                stringify!($rwlock),
                file!(),
                line!()
            ),
        }
    }};
}

#[macro_export]
macro_rules! lock_w {
    ($rwlock:expr) => {{
        match $rwlock.write() {
            Ok(guard) => guard,
            Err(_) => panic!(
                "Failed to acquire write lock on {} at {}:{}",
                stringify!($rwlock),
                file!(),
                line!()
            ),
        }
    }};
}

pub fn toggle_macos_accessory_mode(enabled: bool) {
    #[cfg(target_os = "macos")]
    {
        use crate::get_app_handle;

        let app = get_app_handle();
        if enabled {
            app.set_activation_policy(tauri::ActivationPolicy::Accessory)
                .expect("Failed to set activation policy to accessory mode");
        } else {
            app.set_activation_policy(tauri::ActivationPolicy::Regular)
                .expect("Failed to set activation policy to regular mode");
        }
    }
}
