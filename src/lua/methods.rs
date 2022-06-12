use std::path::PathBuf;

use mlua::{Lua, MultiValue};

use crate::lua::utils::pretty_print_lvalue;

use super::structures::scripts::{LuaScript, SCRIPTS_MANAGER};

pub fn setup_lua(lua: &Lua) {
    let globals = lua.globals();

    // lua.set_hook(HookTriggers::every_line(), |_lua, debug| {
    //     println!("line {}", debug.curr_line());
    //     Ok(())
    // }).expect("reg hook failed");

    let new_print = lua
        .create_function(|_, items: MultiValue| {
            println!(
                "LUA DEBUG : {}",
                items
                    .iter()
                    .map(|value| format!("{:^6}", pretty_print_lvalue(value, None)))
                    .collect::<Vec<_>>()
                    .join("|")
            );
            Ok(())
        })
        .unwrap();
    globals.set("print", new_print).unwrap();
    globals
        .set(
            "luaScript",
            lua.create_function(|_, s: String| {
                let p = LuaScript { name: s, bytecode_fn:None };
                Ok(p)
            })
            .unwrap(),
        )
        .expect("unable to register proj");
    globals
        .set("scriptManager", SCRIPTS_MANAGER.clone())
        .unwrap();
}

pub fn load_script(lua: &Lua, code: String) {
    let cnk = lua.load(&code);
    match cnk.exec() {
        Ok(_) => {}
        Err(e) => println!("{}", e),
    }
}
