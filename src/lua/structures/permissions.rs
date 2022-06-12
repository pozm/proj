use core::fmt;
use std::{sync::{Arc, Mutex}, fmt::Display};
use std::error::Error;
use mlua::{UserData, LuaSerdeExt,ExternalError};
use serde::{Serialize, Deserialize};
use native_dialog;
use mlua::prelude::*;

#[derive(Serialize,Deserialize,Debug,Clone,PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    Fs(String),
    Http(String),
    Script(String),
    Command(String),
}

impl Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::Fs(s) => write!(f, "file system file @ {}", s),
            Permission::Http(s) => write!(f, "access domain \"{}\"", s),
            Permission::Script(s) => write!(f, "script \"{}\"", s),
            Permission::Command(s) => write!(f, "command \"{}\"", s),
        }
    }
}

impl UserData for Permission {}

#[derive(Serialize,Deserialize,Debug,Clone,Default)]

pub struct Permissions{
    pub allowed: Vec<Permission>,
    pub denied: Vec<Permission>,
}

#[derive(Debug)]
struct PermissionError(Permission);

impl fmt::Display for PermissionError {
	fn fmt<'a>(&self, f: &'a mut fmt::Formatter) -> fmt::Result {
		f.write_str(&format!("Permission Error ({})",self.0))
	}
}

impl Error for PermissionError {

}

impl Permissions {
    pub fn ask_for_access<'h>(&mut self,p:&'h Permission) -> LuaResult<()> {

        if self.is_allowed(&p) {
            return Ok(())
        }
        else if self.is_denied(&p) {
            return Err(mlua::Error::ExternalError(Arc::new(PermissionError(p.clone()))))
        }

        if let Ok(allowed) = native_dialog::MessageDialog::new()
        .set_title("Permission")
        .set_text(&format!("The script wants to access\n{}.\ndo you want to grant access?",p.to_string()))
        .show_confirm() && allowed {
            self.allowed.push(p.clone());

        } else {
            self.denied.push(p.clone());
            return Err(mlua::Error::ExternalError(Arc::new(PermissionError(p.clone()))))
        }

        Ok(())
    }
    #[inline]
    pub fn is_allowed(&self,p:&Permission) -> bool {
        self.allowed.iter().any( |x:&Permission| {
            match (x,&p) {
                (Permission::Fs(x), Permission::Fs(p)) => p.starts_with(x),
                (Permission::Http(x), Permission::Http(p)) => p.starts_with(x),
                (Permission::Command(x), Permission::Command(p)) => p.starts_with(x),
                (Permission::Script(x), Permission::Script(p)) => p.starts_with(x),
                _=>false
            }
        })
    }
    #[inline]
    pub fn is_denied(&self,p:&Permission) -> bool {
        self.denied.iter().any( |x:&Permission| {
            match (x,&p) {
                (Permission::Fs(x), Permission::Fs(p)) => p.starts_with(x),
                (Permission::Http(x), Permission::Http(p)) => p.starts_with(x),
                (Permission::Command(x), Permission::Command(p)) => p.starts_with(x),
                (Permission::Script(x), Permission::Script(p)) => p.starts_with(x),
                _=>false
            }
        })
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
