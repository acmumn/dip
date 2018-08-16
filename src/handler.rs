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
    program: Program,
}

pub enum Program {
    Command(String),
    Exec(PathBuf),
}

impl Handler {
    pub fn from(config: &TomlValue) -> Result<Self, Error> {
        let handler = config
            .get("type")
            .ok_or(err_msg("No 'type' found."))?
            .as_str()
            .ok_or(err_msg("'type' is not a string."))?;
        let program = match handler {
            "script" => {
                let command = config
                    .get("command")
                    .ok_or(err_msg("No 'command' found"))?
                    .as_str()
                    .ok_or(err_msg("'command' is not a string."))?;
                Program::Command(command.to_owned())
            }
            handler => {
                let programs = PROGRAMS.lock().unwrap();
                let program = programs
                    .get(handler)
                    .ok_or(err_msg(format!("'{}' is not a valid executable", handler)))
                    .map(|value| value.clone())?;
                Program::Exec(program)
            }
        };
        let config = config.clone();
        Ok(Handler { config, program })
    }
    pub fn run(&self, input: JsonValue) -> Result<JsonValue, Error> {
        let config = {
            let mut buf: Vec<u8> = Vec::new();
            {
                let mut serializer = JsonSerializer::new(&mut buf);
                TomlValue::serialize(&self.config, &mut serializer)?;
            }
            String::from_utf8(buf).unwrap()
        };

        match &self.program {
            Program::Command(ref cmd) => {
                // TODO: allow some kind of simple variable replacement
                let output = Command::new("/bin/bash").arg("-c").arg(cmd).output()?;
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
            }
            Program::Exec(ref path) => {
                let mut child = Command::new(&path)
                    .env("DIP_ROOT", "")
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
            }
        };
        Ok(json!({}))
    }
}
