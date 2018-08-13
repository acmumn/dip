extern crate failure;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate notify;
extern crate regex;
extern crate walkdir;

mod handler;
mod hook;

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use failure::Error;
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use walkdir::WalkDir;

use handler::*;
use hook::*;

lazy_static! {
    static ref URIPATTERN: Regex =
        Regex::new(r"/webhook/(?P<name>[A-Za-z_][A-Za-z0-9_]*)").unwrap();
    static ref HANDLERS: HashMap<String, Box<Handler>> = HashMap::new();
    static ref HOOKS: HashMap<String, Box<Hook>> = HashMap::new();
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
    let handler = match HOOKS.get(name) {
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

fn watch<P>(root: P) -> notify::Result<()>
where
    P: AsRef<Path>,
{
    let (tx, rx) = mpsc::channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;
    println!("Watching {:?}", root.as_ref().to_path_buf());
    watcher.watch(root, RecursiveMode::Recursive)?;
    loop {
        match rx.recv() {
            Ok(event) => println!("{:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

pub fn run<P>(root: P) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let root = root.as_ref().to_path_buf();
    let handlers_dir = {
        let mut p = root.clone();
        p.push("handlers");
        p
    };
    if handlers_dir.exists() {
        for entry in WalkDir::new(handlers_dir) {
            let path = entry?.path().to_path_buf();
            println!("{:?}", path);
        }
    }

    thread::spawn(|| watch(root));

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr)
        .serve(|| service_fn_ok(service_fn_wrapper))
        .map_err(|e| eprintln!("server error: {}", e));
    hyper::rt::run(server);
    Ok(())
}
