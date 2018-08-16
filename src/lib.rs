//! # Dip

#[macro_use]
extern crate failure;
extern crate futures;
extern crate hyper;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
extern crate notify;
#[macro_use]
extern crate structopt;
extern crate regex;
extern crate toml;
extern crate walkdir;

pub mod config;
pub mod handler;
pub mod hook;
pub mod service;

use std::collections::HashMap;
use std::net::SocketAddrV4;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::Duration;

use failure::{err_msg, Error};
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::Server;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use walkdir::WalkDir;

pub use config::Config;
pub use handler::*;
use hook::*;
use service::*;

const URIPATTERN_STR: &str = r"/webhook/(?P<name>[A-Za-z._][A-Za-z0-9._]*)";

lazy_static! {
    static ref URIPATTERN: Regex = Regex::new(URIPATTERN_STR).unwrap();
    // static ref HANDLERS: Mutex<HashMap<String, Box<Handler>>> = Mutex::new(HashMap::new());
    static ref PROGRAMS: Mutex<HashMap<String, PathBuf>> = Mutex::new(HashMap::new());
    static ref HOOKS: Mutex<HashMap<String, Hook>> = Mutex::new(HashMap::new());
}

const NOTFOUND: &str = "<html> <head> <style> * { font-family: sans-serif; } body { padding: 20px 60px; } </style> </head> <body> <h1>Looks like you took a wrong turn!</h1> <p>There's nothing to see here.</p> </body> </html>";

fn load_config<P>(root: P)
where
    P: AsRef<Path>,
{
    println!("Reloading config...");
    // hold on to the lock while config is being reloaded
    {
        let mut programs = PROGRAMS.lock().unwrap();
        // TODO: some kind of smart diff
        programs.clear();
        let programs_dir = {
            let mut p = root.as_ref().to_path_buf();
            p.push("handlers");
            p
        };
        if programs_dir.exists() {
            for entry in WalkDir::new(programs_dir) {
                let path = match entry.as_ref().map(|e| e.path()) {
                    Ok(path) => path,
                    _ => continue,
                };
                if !path.is_file() {
                    continue;
                }
                match path.file_name()
                    .and_then(|s| s.to_str())
                    .ok_or(err_msg("???"))
                    .map(|s| {
                        let filename = s.to_owned();
                        programs.insert(filename, path.to_path_buf())
                    }) {
                    _ => (), // don't care
                }
            }
        }
    }
    {
        let mut hooks = HOOKS.lock().unwrap();
        hooks.clear();
        let hooks_dir = {
            let mut p = root.as_ref().to_path_buf();
            p.push("hooks");
            p
        };
        if hooks_dir.exists() {
            for entry in WalkDir::new(hooks_dir) {
                let path = match entry.as_ref().map(|e| e.path()) {
                    Ok(path) => path,
                    _ => continue,
                };
                if !path.is_file() {
                    continue;
                }
                match (|path: &Path| -> Result<(), Error> {
                    let hook = Hook::from_file(path)?;
                    let name = hook.get_name();
                    hooks.insert(name, hook);
                    Ok(())
                })(path)
                {
                    Ok(_) => (),
                    Err(err) => eprintln!("Failed to read config from {:?}: {}", path, err),
                }
            }
        }
    }
}

fn watch<P>(root: P) -> notify::Result<()>
where
    P: AsRef<Path>,
{
    let (tx, rx) = mpsc::channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;
    println!("Watching {:?}", root.as_ref().to_path_buf());
    watcher.watch(root.as_ref(), RecursiveMode::Recursive)?;
    loop {
        match rx.recv() {
            Ok(_) => {
                // for now, naively reload entire config every time
                // TODO: don't do this
                load_config(root.as_ref())
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

/// Main entry point of the entire application.
pub fn run(config: &Config) -> Result<(), Error> {
    load_config(&config.root);

    let v = config.root.clone();
    thread::spawn(|| watch(v));

    let addr: SocketAddrV4 = SocketAddrV4::from_str(config.bind.as_ref())?;
    let server = Server::bind(&addr.into())
        .serve(|| service_fn(dip_service))
        .map_err(|e| eprintln!("server error: {}", e));
    println!("Listening on {:?}", addr);
    hyper::rt::run(server);
    Ok(())
}
