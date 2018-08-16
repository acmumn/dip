use std::path::PathBuf;
use std::process::Command;

use failure::{err_msg, Error};
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

use PROGRAMS;

pub struct Handler {
    exec: PathBuf,
}

impl Handler {
    pub fn from(config: &TomlValue) -> Result<Self, Error> {
        let handler = config
            .get("type")
            .ok_or(err_msg("No 'type' found."))?
            .as_str()
            .ok_or(err_msg("'type' is not a string."))?;
        let exec = {
            let programs = PROGRAMS.lock().unwrap();
            programs
                .get(handler)
                .ok_or(err_msg(format!("'{}' is not a valid executable", handler)))
                .map(|value| value.clone())?
        };
        Ok(Handler { exec })
    }
    pub fn run(&self, _: Result<JsonValue, Error>) -> Result<JsonValue, Error> {
        Command::new(&self.exec)
            .env("DIP_ROOT", "")
            .output()
            .map_err(|err| err_msg(format!("{}", err)))
            .and_then(|output| {
                if !output.status.success() {
                    return Err(err_msg(format!(
                        "'{:?}' returned with a non-zero status code: {}",
                        self.exec, output.status
                    )));
                }
                Ok(json!({}))
            })
    }
}
