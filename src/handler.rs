use std::fmt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use failure::{err_msg, Error};
use futures::{
    future::{self, Either},
    Future,
};
use serde::Serialize;
use serde_json::{Serializer as JsonSerializer, Value as JsonValue};
use tokio::io::write_all;
use tokio_process::CommandExt;
use toml::Value as TomlValue;

use github;

/// A single instance of handler as defined by the config.
#[derive(Clone, Debug)]
pub struct Handler {
    pub(crate) config: TomlValue,
    pub(crate) action: Action,
}

/// Describes an action that a hook can take.
#[derive(Clone)]
pub enum Action {
    /// A builtin function (for example, the Github handler).
    Builtin(fn(&Environment, &TomlValue, &JsonValue) -> Result<JsonValue, Error>),
    /// A command represents a string to be executed by `bash -c`.
    Command(String),
    /// A program represents one of the handlers specified in the `handlers` directory.
    Program(String),
}

/// Describes the environment for running a builtin.
#[derive(Clone)]
pub struct Environment {
    /// The current working directory.
    pub workdir: PathBuf,
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Action::Builtin(_) => write!(f, "Builtin"),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl Handler {
    pub(crate) fn from(config: &TomlValue) -> Result<Self, Error> {
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
            "github" => Action::Builtin(github::main),
            handler => Action::Program(handler.to_owned()),
        };
        let config = config.clone();
        Ok(Handler { config, action })
    }

    /// Runs the given [action](Action) and produces a [Future](Future).
    pub fn run(
        config: TomlValue,
        action: Action,
        temp_path: PathBuf,
        input: JsonValue,
    ) -> impl Future<Item = (PathBuf, JsonValue), Error = Error> {
        let temp_path_cp = temp_path.clone();
        let config_str = {
            let mut buf: Vec<u8> = Vec::new();
            {
                let mut serializer = JsonSerializer::new(&mut buf);
                TomlValue::serialize(&config, &mut serializer).unwrap();
            }
            String::from_utf8(buf).unwrap()
        };

        let command_helper = move |command: &mut Command| {
            command
                .current_dir(&temp_path)
                .env("DIP_ROOT", "lol")
                .env("DIP_WORKDIR", &temp_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
        };

        let output: Box<Future<Item = JsonValue, Error = Error> + Send> = match action {
            Action::Builtin(ref func) => {
                let workdir = temp_path_cp.clone();
                let env = Environment { workdir };
                let result = func(&env, &config, &input);
                Box::new(future::result(result))
            }
            Action::Command(ref cmd) => {
                // TODO: allow some kind of simple variable replacement
                let mut command = Command::new("/bin/bash");
                command_helper(&mut command);
                let child = command.arg("-c").arg(cmd);
                let result = child
                    .output_async()
                    .map_err(|err| err_msg(format!("failed to spawn child: {}", err)))
                    .and_then(|output| {
                        let stdout =
                            String::from_utf8(output.stdout).unwrap_or_else(|_| String::new());
                        let stderr =
                            String::from_utf8(output.stderr).unwrap_or_else(|_| String::new());
                        future::ok(json!({
                            "stdout": stdout,
                            "stderr": stderr,
                        }))
                    });
                Box::new(result)
            }
            Action::Program(ref path) => {
                let mut command = Command::new(&path);
                command_helper(&mut command);
                let mut child = command
                    .arg("--config")
                    .arg(config_str)
                    .spawn_async()
                    .expect("could not spawn child");

                let stdin = child.stdin().take().unwrap();

                let input = format!("{}", input);
                let result = write_all(stdin, input)
                    .and_then(|_| child.wait_with_output())
                    .map_err(|err| err_msg(format!("error: {}", err)))
                    .and_then(|output| {
                        let stdout =
                            String::from_utf8(output.stdout).unwrap_or_else(|_| String::new());
                        let stderr =
                            String::from_utf8(output.stderr).unwrap_or_else(|_| String::new());
                        if output.status.success() {
                            Either::A(future::ok(json!({
                                "stdout": stdout,
                                "stderr": stderr,
                            })))
                        } else {
                            Either::B(future::err(err_msg(format!(
                                "Failed, stdout: '{}', stderr: '{}'",
                                stdout, stderr
                            ))))
                        }
                    });

                Box::new(result)
            }
        };
        output.map(|x| (temp_path_cp, x))
    }
}
