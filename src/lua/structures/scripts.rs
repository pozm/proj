use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

use mlua::{Function, RegistryKey, UserData};

#[derive(Default, Clone, Debug)]
pub struct LuaScript {
    pub name: String,
}
impl UserData for LuaScript {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_set("invoke_fn", |lua, this, f: Function<'_>| {
            SCRIPTS_MANAGER
                .lock()
                .unwrap()
                .fns
                .insert(this.name.to_string(), Some(lua.create_registry_value(f)?));
            Ok(())
        });
        fields.add_meta_field_with("__name", |lua| Ok("LuaScript".to_string()));
    }
}
#[derive(Default, Debug)]
pub struct ScriptsManager {
    pub scripts: Vec<LuaScript>,
    pub fns: HashMap<String, Option<RegistryKey>>,
}

impl UserData for ScriptsManager {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("add", |l, this, s: LuaScript| {
            // if !s.is::<LuaScript>() {
            //     return Err(mlua::Error::RuntimeError("Expected LuaScript".to_string()));
            // }

            this.scripts.push(s);
            Ok(())
        })
    }
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("scripts", |l, t| {
            Ok(t.scripts
                .iter()
                .map(|s| &s.name)
                .cloned()
                .collect::<Vec<String>>())
        });

        fields.add_meta_field_with("__name", |lua| Ok("ScriptsManager".to_string()));
    }
}
// impl ScriptsManager {
//     const INSTANCE : Self = Self {
//         scripts: vec![],
//         fns: HashMap::new()
//     };
// }

lazy_static::lazy_static! {
    pub static ref SCRIPTS_MANAGER : Arc<Mutex<ScriptsManager>> = Arc::new(Mutex::new(ScriptsManager { ..Default::default() }));
}
