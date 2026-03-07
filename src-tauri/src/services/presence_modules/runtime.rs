use mlua::{Lua, LuaSerdeExt, UserData, UserDataMethods, Value};
use std::{path::Path, thread, time::Duration};
use tokio::runtime::Runtime;
use tracing::{error, info, warn};

use crate::models::event_payloads::{UserStatusPayload, UserStatusState};
use crate::services::ws::user_status::report_user_status;
use crate::services::ws::{ws_emit_soft, WS_EVENT};

use super::models::PresenceStatus;

pub struct Engine;

impl UserData for Engine {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("log", |_, _, message: String| {
            info!("{}", message);
            Ok(())
        });
        methods.add_method("sleep", |_, _, seconds: u64| {
            thread::sleep(Duration::from_secs(seconds));
            Ok(())
        });
        methods.add_method("path_from_relative", |lua, _, relative_path: String| {
            let script_path: String = lua.globals().get("script_path")?;
            let path = Path::new(&script_path);
            let parent = path.parent().unwrap_or(path);
            let full_path = parent.join(&relative_path);
            Ok(full_path.to_string_lossy().to_string())
        });
        methods.add_method("get_platform", |_, _, ()| Ok(std::env::consts::OS));
        methods.add_method("update_status", |lua, _, value: Value| {
            let status: PresenceStatus = lua.from_value(value)?;
            let rt = Runtime::new().unwrap();
            rt.block_on(update_status(status));
            Ok(())
        });
        methods.add_async_method("update_status_async", |lua, _, value: Value| async move {
            let status: PresenceStatus = lua.from_value(value)?;
            update_status_async(status).await;
            Ok(())
        });
    }
}

async fn update_status(status: PresenceStatus) {
    let user_status = UserStatusPayload {
        presence_status: status,
        state: UserStatusState::Idle,
    };
    report_user_status(user_status).await;
}

async fn update_status_async(status: PresenceStatus) {
    let payload = UserStatusPayload {
        presence_status: status,
        state: UserStatusState::Idle,
    };
    if let Err(e) = ws_emit_soft(WS_EVENT::CLIENT_REPORT_USER_STATUS, payload).await {
        warn!("User status report failed: {}", e);
    }
}

pub fn spawn_lua_runtime(script_path: &Path) -> thread::JoinHandle<()> {
    let script_path = script_path.to_owned();

    thread::spawn(move || {
        let lua = unsafe { Lua::unsafe_new() };
        let globals = lua.globals();

        // Set script_path for engine methods to access
        if let Err(e) = globals.set("script_path", script_path.to_string_lossy().to_string()) {
            error!("Failed to set script_path global: {}", e);
            return;
        }

        let script = match std::fs::read_to_string(&script_path) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to read script: {}", e);
                return;
            }
        };

        if let Err(e) = globals.set("engine", Engine) {
            error!("Failed to set engine global: {}", e);
            return;
        }

        if let Err(e) = lua.load(&script).exec() {
            error!("Failed to execute lua script: {}", e);
        }
    })
}

pub fn spawn_lua_runtime_from_path(path: &Path) -> Result<thread::JoinHandle<()>, std::io::Error> {
    Ok(spawn_lua_runtime(path))
}
