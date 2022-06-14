use mlua::{Error, UserData};
use path_absolutize::*;
use zip::{ZipArchive, write::FileOptions, ZipWriter};
use core::fmt;
use std::{
    fs::{self, create_dir, read_dir, File, OpenOptions},
    io::{Read, Seek, Write, Cursor},
    path::{Path, PathBuf}, sync::Arc,
};
use std::error::Error as OtherError;
use mlua::prelude::*;

use crate::lua::structures::permissions::{PERMISSIONS_MANAGER, Permission};

#[derive(Debug)]
struct FsError(String);

impl fmt::Display for FsError {
	fn fmt<'a>(&self, f: &'a mut fmt::Formatter) -> fmt::Result {
		f.write_str(&format!("FileSystem Error ({})",self.0))
	}
}

impl OtherError for FsError {

}

pub struct LuaFs();

pub struct LuaFile(pub String, pub File);

#[inline]
fn is_path_allowed<T: Into<PathBuf>>(path: T) -> LuaResult<()> {
    let path: &PathBuf = &path.into();

    println!("check if path is allowed: {:?}", path);

    let mut permissions = PERMISSIONS_MANAGER.lock().unwrap();
    let p = Permission::Fs(path.absolutize().unwrap().display().to_string());
    permissions.ask_for_access(&p)


}

impl UserData for LuaFile {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_l, t| Ok(t.0.clone()));
        // fields.add_field_method_get("size", |l,t|Ok(t.1.stream_len()?.clone()))
        fields.add_meta_field_with("__name", |_lua| Ok("LuaFile".to_string()));
    }
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("write", |_l, t, content: String| {
            t.1.write_all(content.as_bytes())?;
            Ok(())
        });
        methods.add_method_mut("read", |_l, t, ()| {
            let mut content = String::new();
            let stream_pos = t.1.stream_position()?;
            t.1.seek(std::io::SeekFrom::Start(0))?;
            t.1.read_to_string(&mut content)?;
            t.1.seek(std::io::SeekFrom::Start(stream_pos))?;
            Ok(content)
        });
        methods.add_method_mut("clear", |_l, t, ()| {
            t.1.set_len(0)?;
            t.1.flush()?;
            t.1.rewind()?;
            Ok(())
        });
        methods.add_method_mut("seek", |_, t, pos: u64| {
            t.1.seek(std::io::SeekFrom::Start(pos))?;
            Ok(())
        });
        methods.add_method_mut("unzip", |_, t, to:String| {
            let path = Path::new(&to).absolutize()?;
            is_path_allowed(path.as_ref())?;
            
            let mut content = vec![];
            let stream_pos = t.1.stream_position()?;
            t.1.seek(std::io::SeekFrom::Start(0))?;
            t.1.read_to_end(&mut content)?;
            t.1.seek(std::io::SeekFrom::Start(stream_pos))?;

            let mut read = Cursor::new(&content);

            let mut z = ZipArchive::new(&mut read).or_else(|e| Err(Error::ExternalError(Arc::new(e))))?;

            z.extract(&path).or_else(|e| Err(Error::ExternalError(Arc::new(e))))?;

            Ok(())
        })
    }
}

impl UserData for LuaFs {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_meta_field_with("__name", |_lua| Ok("LuaFileSystem".to_string()));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("createFile", |_l, t, p: String| {
            let path = Path::new(&p).absolutize()?;
            is_path_allowed(path.as_ref())?;

            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(&path)?;
            let file = LuaFile(path.display().to_string(), file);
            Ok(file)
        });
        methods.add_method("createDir", |_l, t, p: String| {
            let path = Path::new(&p).absolutize()?;
            is_path_allowed(path.as_ref())?;
            create_dir(&path)?;
            // let file = LuaFile(path.display().to_string(), file);

            Ok(path.as_ref().display().to_string())
        });
        methods.add_method("openDir", |_l, t, p: String| {
            let path = Path::new(&p).absolutize()?;
            is_path_allowed(path.as_ref())?;

            let dir = read_dir(path)?;
            // let file = LuaFile(path.display().to_string(), file);

            Ok(dir
                .map(|f| f.unwrap().path().display().to_string())
                .collect::<Vec<_>>())
        });
        methods.add_method("openFile", |_l, t, p: String| {
            let path = Path::new(&p).absolutize()?;
            is_path_allowed(path.as_ref())?;

            let file = OpenOptions::new()
                .create(false)
                .write(true)
                .read(true)
                .open(&path)?;
            let file = LuaFile(path.display().to_string(), file);
            Ok(file)
        });
        methods.add_method("exists", |_l, t, p: String| {
            let path = Path::new(&p).absolutize()?;
            is_path_allowed(path.as_ref())?;

            Ok(path.exists())
        });
        methods.add_method("copy", |_l, t, (fp, tp): (String, String)| {
            let path = Path::new(&tp).absolutize()?;
            is_path_allowed(path.as_ref())?;

            crate::utils::copy(fp, path)?;
            Ok(())
        });
        methods.add_method("move", |_l, t, (fp, tp): (String, String)| {
            let path = Path::new(&tp).absolutize()?;
            is_path_allowed(path.as_ref())?;
            let pathf = Path::new(&fp).absolutize()?;
            is_path_allowed(path.as_ref())?;

            fs::rename(pathf, path)?;
            Ok(())
        })
    }
}
