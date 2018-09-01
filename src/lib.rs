//! # Dip

extern crate hmac;
extern crate secstr;
extern crate sha1;
#[macro_use]
extern crate serde_derive;
extern crate failure;
extern crate futures;
extern crate generic_array;
extern crate hyper;
extern crate mktemp;
extern crate owning_ref;
extern crate serde;
extern crate tokio;
extern crate tokio_process;
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
pub mod github;
pub mod handler;
pub mod hook;
pub mod service;

use std::collections::HashMap;
use std::net::SocketAddrV4;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use failure::Error;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::Server;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;

pub use config::Config;
pub use handler::*;
use hook::*;
use service::*;

const URIPATTERN_STR: &str = r"/webhook/(?P<name>[A-Za-z._][A-Za-z0-9._]*)";

lazy_static! {
    static ref URIPATTERN: Regex = Regex::new(URIPATTERN_STR).unwrap();
    static ref PROGRAMS: Arc<Mutex<HashMap<String, PathBuf>>> =
        Arc::new(Mutex::new(HashMap::new()));
    static ref HOOKS: Arc<Mutex<HashMap<String, Hook>>> = Arc::new(Mutex::new(HashMap::new()));
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
                config::load_config(root.as_ref())
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }
}

/// Main entry point of the entire application.
pub fn run(config: &Config) -> Result<(), Error> {
    config::load_config(&config.root);

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
