//! Configuration.

use std::default::Default;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use failure::{err_msg, Error};
use notify::{self, RecommendedWatcher, RecursiveMode, Watcher};
use walkdir::WalkDir;

use Hook;
use {HOOKS, PROGRAMS};

/// The configuration to be parsed from the command line.
#[derive(Debug, StructOpt)]
pub struct Config {
    /// The root configuration directory for dip. This argument is required.
    #[structopt(short = "d", long = "root", parse(from_os_str))]
    pub root: PathBuf,
    /// A string containing the address to bind to. This defaults to "0.0.0.0:5000".
    #[structopt(short = "b", long = "bind", default_value = "0.0.0.0:5000")]
    pub bind: String,
    /// If a hook is specified here, it will be triggered manually exactly once and then the
    /// program will exit rather than running as a server.
    #[structopt(short = "h", long = "hook")]
    pub hook: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let root = env::current_dir().unwrap();
        assert!(root.exists());

        let bind = "0.0.0.0:5000".to_owned();
        let hook = None;
        Config { root, bind, hook }
    }
}

pub(crate) fn watch<P>(root: P) -> notify::Result<()>
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
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }
}

/// Load config from the root directory. This is called by the watcher.
pub fn load_config<P>(root: P)
where
    P: AsRef<Path>,
{
    println!("Reloading config...");
    // hold on to the lock while config is being reloaded
    {
        let mut programs = match PROGRAMS.lock() {
            Ok(programs) => programs,
            Err(err) => {
                eprintln!("Could not acquire programs lock: {}", err);
                return;
            }
        };
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
                match path
                    .file_name()
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
        let mut hooks = match HOOKS.lock() {
            Ok(hooks) => hooks,
            Err(err) => {
                eprintln!("Could not acquire hooks lock: {}", err);
                return;
            }
        };
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
    println!("Done loading config.");
}
