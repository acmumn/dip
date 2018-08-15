use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::Read;

use failure::{err_msg, Error};
use hyper::{Body, Request, Response};
use toml::Value;
use walkdir::WalkDir;

pub struct Hook {
    handler_type: String,
}

impl Hook {
    pub fn from(config: &Value) -> Result<Self, Error> {
        let handler_type = config
            .get("type")
            .ok_or(err_msg("Missing field 'type'"))?
            .as_str()
            .ok_or(err_msg("Field 'type' is not a string"))?
            .to_owned();
        Ok(Hook { handler_type })
    }
    pub fn handle(&self, payload: &Request<Body>) -> Result<Response<Body>, Error> {
        Ok(Response::new(Body::from("lol")))
    }
}

pub fn load_from_config<P>(path: P, hooks: &mut HashMap<String, Hook>) where P: AsRef<Path> {
        let hooks_dir = {
            let mut p = path.as_ref().to_path_buf();
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
                    println!("Added hook '{}'", filename);
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