use std::sync::{Arc, Mutex};

use mlua::{UserData, LuaSerdeExt};
use serde::{Serialize, Deserialize};


#[derive(Serialize,Deserialize,Debug,Clone,PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    Fs(String),
    Http(String),
    Script(String),
    Command(String),
}

impl UserData for Permission {}

#[derive(Serialize,Deserialize,Debug,Clone,Default)]

pub struct Permissions{
    pub allowed: Vec<Permission>,
    pub denied: Vec<Permission>,
}

impl Permissions {
    pub fn ask_for_access(&mut self,p:Permission) -> Result<(),()> {
        self.allowed.push(p);
        Ok(())
    }
}

impl UserData for Permissions {

    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("allowed", |l,t| {
            l.to_value(&t)
        });
        
        fields.add_meta_field_with("__name", |_lua| Ok("Permissions".to_string()));
    }

}

lazy_static::lazy_static! {
    pub static ref PERMISSIONS_MANAGER : Arc<Mutex<Permissions>> = Arc::new(Mutex::new(Permissions { ..Default::default() }));
}
