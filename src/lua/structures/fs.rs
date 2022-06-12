use mlua::{Error, UserData};
use path_absolutize::*;
use std::{
    fs::{self, create_dir, read_dir, File, OpenOptions},
    io::{Read, Seek, Write},
    path::{Path, PathBuf},
};

pub struct LuaFs(pub Vec<String>);

pub struct LuaFile(pub String, pub File);

impl LuaFs {
    fn is_path_allowed<T: Into<PathBuf>>(&self, path: T) -> bool {
        let path: &PathBuf = &path.into();

        println!("check if path is allowed: {:?}", path);

        self.0
            .iter()
            .any(|allowed_path| path.starts_with(allowed_path))
    }
}
impl UserData for LuaFile {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |l, t| Ok(t.0.clone()));
        // fields.add_field_method_get("size", |l,t|Ok(t.1.stream_len()?.clone()))
        fields.add_meta_field_with("__name", |lua| Ok("LuaFile".to_string()));
    }
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("write", |l, t, (content): (String)| {
            t.1.write_all(content.as_bytes())?;
            Ok(())
        });
        methods.add_method_mut("read", |l, t, ()| {
            let mut content = String::new();
            let stream_pos = t.1.stream_position()?;
            t.1.seek(std::io::SeekFrom::Start(0))?;
            t.1.read_to_string(&mut content)?;
            t.1.seek(std::io::SeekFrom::Start(stream_pos))?;
            Ok(content)
        });
        methods.add_method_mut("clear", |l, t, ()| {
            t.1.set_len(0)?;
            t.1.flush()?;
            t.1.rewind()?;
            Ok(())
        });
        methods.add_method_mut("seek", |_, t, pos: u64| {
            t.1.seek(std::io::SeekFrom::Start(pos))?;
            Ok(())
        })
    }
}

impl UserData for LuaFs {
    // fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
    //     fields.add_field_method_get("create_file", |l,t| {
    //         l.create_function(|l,(t,p):(LuaFs,String)| {
    //             let path = Path::new(&p);
    //             if !t.is_path_allowed(path) {

    //             }

    //             Ok("")
    //         }).unwrap().bind(*t)
    //     })
    // }
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_meta_field_with("__name", |lua| Ok("LuaFileSystem".to_string()));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("createFile", |l, t, p: String| {
            let path = Path::new(&p).absolutize()?;
            if !t.is_path_allowed(path.as_ref()) {
                return Err(Error::RuntimeError("Path not allowed".to_string()));
            }

            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(&path)?;
            let file = LuaFile(path.display().to_string(), file);
            Ok(file)
        });
        methods.add_method("createDir", |l, t, p: String| {
            let path = Path::new(&p);
            if !t.is_path_allowed(path) {
                return Err(Error::RuntimeError("Path not allowed".to_string()));
            }
            create_dir(path)?;
            // let file = LuaFile(path.display().to_string(), file);

            Ok(path.display().to_string())
        });
        methods.add_method("openDir", |l, t, p: String| {
            let path = Path::new(&p);
            if !t.is_path_allowed(path) {
                return Err(Error::RuntimeError("Path not allowed".to_string()));
            }

            let dir = read_dir(path)?;
            // let file = LuaFile(path.display().to_string(), file);

            Ok(dir
                .map(|f| f.unwrap().path().display().to_string())
                .collect::<Vec<_>>())
        });
        methods.add_method("openFile", |_l, t, p: String| {
            let path = Path::new(&p).absolutize()?;
            if !t.is_path_allowed(path.as_ref()) {
                return Err(Error::RuntimeError("Path not allowed".to_string()));
            }

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
            if !t.is_path_allowed(path.as_ref()) {
                return Err(Error::RuntimeError("Path not allowed".to_string()));
            }

            Ok(path.exists())
        });
        methods.add_method("copy", |_l, t, (fp, tp): (String, String)| {
            let path = Path::new(&tp).absolutize()?;
            if !t.is_path_allowed(path.as_ref()) {
                return Err(Error::RuntimeError("Path not allowed".to_string()));
            }

            crate::utils::copy(fp, path)?;
            Ok(())
        });
        methods.add_method("move", |_l, t, (fp, tp): (String, String)| {
            let path = Path::new(&tp).absolutize()?;
            if !t.is_path_allowed(path.as_ref()) {
                return Err(Error::RuntimeError("Path not allowed".to_string()));
            }
            let pathf = Path::new(&fp).absolutize()?;
            if !t.is_path_allowed(pathf.as_ref()) {
                return Err(Error::RuntimeError("Path not allowed".to_string()));
            }

            fs::rename(pathf, path)?;
            Ok(())
        })
    }
}
