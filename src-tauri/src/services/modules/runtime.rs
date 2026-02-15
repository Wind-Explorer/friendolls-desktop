use mlua::{Lua, UserData, UserDataMethods};
use std::{path::Path, thread, time::Duration};
use tracing::{error, info};

pub struct Engine;

impl UserData for Engine {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("log", |_, _, message: String| {
            info!("{}", message);
            Ok(())
        });
        methods.add_method("sleep", |_, _, seconds: u64| {
            thread::sleep(Duration::from_secs(seconds));
            Ok(())
        });
    }
}

fn load_script(path: &Path) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

fn setup_engine_globals(lua: &Lua) -> Result<(), mlua::Error> {
    let globals = lua.globals();
    globals.set("engine", Engine)
}

pub fn spawn_lua_runtime(script: &str) -> thread::JoinHandle<()> {
    let script = script.to_string();

    thread::spawn(move || {
        let lua = Lua::new();

        if let Err(e) = setup_engine_globals(&lua) {
            error!("Failed to set engine global: {}", e);
            return;
        }

        if let Err(e) = lua.load(&script).exec() {
            error!("Failed to execute lua script: {}", e);
            return;
        }
    })
}

pub fn spawn_lua_runtime_from_path(path: &Path) -> Result<thread::JoinHandle<()>, std::io::Error> {
    let script = load_script(path)?;
    Ok(spawn_lua_runtime(&script))
}
