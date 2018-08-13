extern crate failure;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate notify;
extern crate regex;
extern crate toml;
extern crate walkdir;

mod handler;
mod hook;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::Duration;

use failure::{err_msg, Error};
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use toml::Value;
use walkdir::WalkDir;

use handler::*;
use hook::*;

lazy_static! {
    static ref URIPATTERN: Regex =
        Regex::new(r"/webhook/(?P<name>[A-Za-z_][A-Za-z0-9_]*)").unwrap();
    static ref HANDLERS: HashMap<String, Box<Handler>> = HashMap::new();
    static ref HOOKS: Mutex<HashMap<String, Hook>> = Mutex::new(HashMap::new());
}

const NOTFOUND: &str = r#"<html>
    <head>
        <style>
            * { font-family: sans-serif; }
            body { padding: 20px 60px; }
        </style>
    </head>
    <body>
        <h1>Looks like you took a wrong turn!</h1>
        <p>There's nothing to see here.</p>
    </body>
</html>"#;

fn service_fn(req: Request<Body>) -> Option<Response<Body>> {
    let captures = match URIPATTERN.captures(req.uri().path()) {
        Some(value) => value,
        None => return None,
    };
    let name = match captures.name("name") {
        Some(name) => name.as_str(),
        None => return None,
    };
    let hooks = HOOKS.lock().unwrap();
    let handler = match hooks.get(name) {
        Some(handler) => handler,
        None => return None,
    };
    handler.handle(&req)
}

fn service_fn_wrapper(req: Request<Body>) -> Response<Body> {
    match service_fn(req) {
        Some(response) => response,
        None => Response::new(Body::from(NOTFOUND)),
    }
}

fn load_config<P>(root: P)
where
    P: AsRef<Path>,
{
    println!("Reloading config...");
    // hold on to the lock while config is being reloaded
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
                let filename = path
                    .file_name()
                    .ok_or(err_msg("what the fuck bro"))?
                    .to_str()
                    .ok_or(err_msg("???"))?
                    .to_owned();
                let mut file = File::open(path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                let config = contents.parse::<Value>()?;
                let hook = Hook::from(&config)?;
                hooks.insert(filename, hook);
                Ok(())
            })(path)
            {
                Ok(_) => (),
                Err(err) => eprintln!("Failed to read config from {:?}: {}", path, err),
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
                load_config(root.as_ref())
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

pub fn run<P>(root: P) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    load_config(&root);

    let v = root.as_ref().to_path_buf();
    thread::spawn(|| watch(v));

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr)
        .serve(|| service_fn_ok(service_fn_wrapper))
        .map_err(|e| eprintln!("server error: {}", e));
    println!("Listening on {:?}", addr);
    hyper::rt::run(server);
    Ok(())
}
