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
        let config = config.clone();
        Ok(Handler { config, exec })
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

        let mut child = Command::new(&self.exec)
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
                self.exec,
                output.status,
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            )));
        }
        Ok(json!({}))
    }
}
