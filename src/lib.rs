//! # Dip
//!
//! The configurable webhook server. Latest stable binary releases for Linux are available on the [releases][1] page.
//!
//! ## Getting Started
//!
//! Setup is incredibly simple: first, obtain a copy of `dip` either through the binary releases page or by compiling from source.
//! Then, create a directory that you'll use as your `DIP_ROOT` directory. It should look like this:
//!
//! ```text
//!
//! ```
//!
//! [1]: https://github.com/acmumn/dip/releases

#![deny(missing_docs)]

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
mod github;
mod handler;
pub mod hook;
mod service;

use std::collections::HashMap;
use std::net::SocketAddrV4;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;

use failure::Error;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::Server;
use regex::Regex;

pub use config::Config;
pub use handler::*;
use hook::*;
use service::*;

const URIPATTERN_STR: &str = r"/webhook/(?P<name>[A-Za-z._][A-Za-z0-9._]*)";

lazy_static! {
    static ref URIPATTERN: Regex =
        Regex::new(URIPATTERN_STR).expect("Could not compile regular expression.");
    static ref PROGRAMS: Arc<Mutex<HashMap<String, PathBuf>>> =
        Arc::new(Mutex::new(HashMap::new()));
    static ref HOOKS: Arc<Mutex<HashMap<String, Hook>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Main entry point of the entire application.
pub fn run(config: &Config) -> Result<(), Error> {
    config::load_config(&config.root);

    let v = config.root.clone();
    thread::spawn(|| config::watch(v));

    let addr: SocketAddrV4 = SocketAddrV4::from_str(config.bind.as_ref())?;
    let server = Server::bind(&addr.into())
        .serve(|| service_fn(dip_service))
        .map_err(|e| eprintln!("server error: {}", e));
    println!("Listening on {:?}", addr);
    hyper::rt::run(server);
    Ok(())
}
