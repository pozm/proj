#![feature(let_chains)]
mod lua;
mod utils;
use std::{
    fs::{create_dir_all, read_dir, File},
    io::Read,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use clap::Parser;
use directories::ProjectDirs;
use lua::structures::{fs::LuaFs, http::LuaHttp, scripts::SCRIPTS_MANAGER, permissions::{PERMISSIONS_MANAGER, Permission}};
use mlua::{Function, Lua, LuaOptions, StdLib};
use path_absolutize::Absolutize;

#[derive(Parser)]
#[clap(author, version, about)]
struct ProgramArgs {
    #[clap(short, long, parse(from_os_str), value_name = "PROJECT")]
    pub project_path: Option<PathBuf>,
    #[clap(short, long)]
    pub script: Option<String>,
    #[clap(short, long)]
    pub list_scripts: bool,
    #[clap(short = 'c', long)]
    pub show_config: bool,
}
#[tokio::main]
async fn main() {
    let cli: ProgramArgs = ProgramArgs::parse();

    if !cli.list_scripts && cli.script.is_none() && !cli.show_config {
        println!("No script specified");
        return;
    }

    let proj_dirs =
        ProjectDirs::from("com", "Pozm", "Proj").expect("Failed to get project directories");
    let proj = proj_dirs.config_dir();

    let scripts_path = proj.join("scripts");
    let scripts_conf_path = scripts_path.join("config");
    create_dir_all(&proj).unwrap();
    create_dir_all(&scripts_path).unwrap();
    if cli.show_config {
        println!("config can be found at {:?}", proj.absolutize().unwrap());
        return;
    }
    let lua = Lua::new_with(
        StdLib::BIT | StdLib::MATH | StdLib::STRING | StdLib::TABLE,
        LuaOptions::default(),
    )
    .expect("unable to make lua");

    lua::methods::setup_lua(&lua);

    let read = read_dir(&scripts_path).expect("unable to open scripts directory");

    for entry in read {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            println!("loading from {}", entry.path().display());
            let mut file = File::open(entry.path()).expect("unable to open file");
            let mut lua_code = String::new();
            file.read_to_string(&mut lua_code);
            lua::methods::load_script(&lua, lua_code)
        }
    }

    // testing

    lua::methods::load_script(&lua, include_str!("../example.proj.lua").to_string());

    // end

    let script_names = SCRIPTS_MANAGER
        .lock()
        .unwrap()
        .scripts
        .iter()
        .map(|x| &x.name)
        .cloned()
        .collect::<Vec<_>>();
    if cli.list_scripts {
        println!("loaded scripts : {}", script_names.join(", "));
        return;
    }
    if cli.project_path.is_none() {
        println!("No project path specified");
        return;
    }

    create_dir_all(cli.project_path.as_ref().unwrap());

    if let Some(script) = cli.script {
        if !script_names.contains(&script) {
            println!("unable to find that script, try using listing scripts")
        } else {
            if let Some(lua_fn) = SCRIPTS_MANAGER.lock().unwrap().fns.get(&script).unwrap() {
                let lua_fn = lua.registry_value::<Function>(lua_fn).unwrap();
                let proj_dir_path = cli
                .project_path
                .as_ref()
                .unwrap()
                .absolutize()
                .unwrap();
                let proj_dir = proj_dir_path
                    .clone()
                    .display()
                    .to_string();

                PERMISSIONS_MANAGER.lock().unwrap().allowed.push(Permission::Fs(proj_dir.clone()));
                // PERMISSIONS_MANAGER.lock().unwrap().denied.push(Permission::Fs(proj_dir_path.join("..").absolutize().unwrap().clone().display().to_string())); // test

                let globs = lua.globals();
                globs
                    .set("DIR_PROJECT", format!("{}/", proj_dir.clone()))
                    .unwrap();
                globs.set("fs", LuaFs()).unwrap();
                globs
                    .set(
                        "http",
                        LuaHttp(Arc::new(Mutex::new(reqwest::Client::new()))),
                    )
                    .unwrap();
                globs.set("permissions", PERMISSIONS_MANAGER.clone()).unwrap();
                match lua_fn.call_async::<_, ()>(()).await {
                    Ok(_) => println!("done!"),
                    Err(e) => eprintln!("error when calling script : {}", e),
                }
            } else {
                println!("the script you provided is broken.")
            }
        }
    }
}
