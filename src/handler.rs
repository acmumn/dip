use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use failure::{err_msg, Error};
use serde::Serialize;
use serde_json::{Serializer as JsonSerializer, Value as JsonValue};
use toml::Value as TomlValue;

use PROGRAMS;

pub struct Handler {
    config: TomlValue,
    action: Action,
}

pub enum Action {
    Command(String),
    Exec(PathBuf),
}

impl Handler {
    pub fn config(&self) -> &TomlValue {
        &self.config
    }
    pub fn from(config: &TomlValue) -> Result<Self, Error> {
        let handler = config
            .get("type")
            .ok_or(err_msg("No 'type' found."))?
            .as_str()
            .ok_or(err_msg("'type' is not a string."))?;
        let action = match handler {
            "command" => {
                let command = config
                    .get("command")
                    .ok_or(err_msg("No 'command' found"))?
                    .as_str()
                    .ok_or(err_msg("'command' is not a string."))?;
                Action::Command(command.to_owned())
            }
            handler => {
                let programs = PROGRAMS.lock().unwrap();
                let program = programs
                    .get(handler)
                    .ok_or(err_msg(format!("'{}' is not a valid executable", handler)))
                    .and_then(|value| {
                        value
                            .canonicalize()
                            .map_err(|_| err_msg("failed to canonicalize the path"))
                    })
                    .map(|value| value.clone())?;
                Action::Exec(program)
            }
        };
        let config = config.clone();
        Ok(Handler { config, action })
    }

    pub fn run(&self, temp_path: &PathBuf, input: JsonValue) -> Result<JsonValue, Error> {
        let config = {
            let mut buf: Vec<u8> = Vec::new();
            {
                let mut serializer = JsonSerializer::new(&mut buf);
                TomlValue::serialize(&self.config, &mut serializer)?;
            }
            String::from_utf8(buf).unwrap()
        };

        let output = match &self.action {
            Action::Command(ref cmd) => {
                // TODO: allow some kind of simple variable replacement
                let output = Command::new("/bin/bash")
                    .current_dir(&temp_path)
                    .env("DIP_ROOT", "lol")
                    .env("DIP_WORKDIR", temp_path)
                    .arg("-c")
                    .arg(cmd)
                    .output()?;
                if !output.status.success() {
                    // TODO: get rid of unwraps
                    return Err(err_msg(format!(
                        "Command '{}' returned with a non-zero status code: {}\nstdout:\n{}\nstderr:\n{}",
                        cmd,
                        output.status,
                        String::from_utf8(output.stdout).unwrap_or_else(|_| String::new()),
                        String::from_utf8(output.stderr).unwrap_or_else(|_| String::new())
                    )));
                }
                output
            }
            Action::Exec(ref path) => {
                let mut child = Command::new(&path)
                    .current_dir(&temp_path)
                    .env("DIP_ROOT", "")
                    .env("DIP_WORKDIR", temp_path)
                    .arg("--config")
                    .arg(config)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?;
                {
                    match child.stdin {
                        Some(ref mut stdin) => {
                            write!(stdin, "{}", input)?;
                        }
                        None => bail!("done fucked"),
                    };
                }
                let output = child.wait_with_output()?;
                if !output.status.success() {
                    // TODO: get rid of unwraps
                    return Err(err_msg(format!(
                        "'{:?}' returned with a non-zero status code: {}\nstdout:\n{}\nstderr:\n{}",
                        path,
                        output.status,
                        String::from_utf8(output.stdout).unwrap_or_else(|_| String::new()),
                        String::from_utf8(output.stderr).unwrap_or_else(|_| String::new())
                    )));
                }
                output
            }
        };

        let stdout = String::from_utf8(output.stdout).unwrap_or_else(|_| String::new());
        let stderr = String::from_utf8(output.stderr).unwrap_or_else(|_| String::new());
        Ok(json!({
                "stdout": stdout,
                "stderr": stderr,
            }))
    }
}
