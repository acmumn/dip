use std::path::PathBuf;
use std::process::{Command, Stdio};

use failure::{err_msg, Error};
use futures::{
    future::{self, Either, FutureResult},
    sink::Sink,
    Future,
};
use serde::Serialize;
use serde_json::{Serializer as JsonSerializer, Value as JsonValue};
use tokio::io::write_all;
use tokio_process::CommandExt;
use toml::Value as TomlValue;

use PROGRAMS;

#[derive(Clone, Debug)]
pub struct Handler {
    pub config: TomlValue,
    pub action: Action,
}

#[derive(Clone, Debug)]
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
                    }).map(|value| value.clone())?;
                Action::Exec(program)
            }
        };
        let config = config.clone();
        Ok(Handler { config, action })
    }

    pub fn run(
        config: TomlValue,
        action: Action,
        temp_path: PathBuf,
        input: JsonValue,
    ) -> impl Future<Item = (PathBuf, JsonValue), Error = Error> {
        println!("Running: {:?} :: {:?}", config, action);
        let config = {
            let mut buf: Vec<u8> = Vec::new();
            {
                let mut serializer = JsonSerializer::new(&mut buf);
                TomlValue::serialize(&config, &mut serializer).unwrap();
            }
            String::from_utf8(buf).unwrap()
        };

        let output: Box<Future<Item = JsonValue, Error = Error> + Send> = match action {
            Action::Command(ref cmd) => {
                // TODO: allow some kind of simple variable replacement
                let mut child = Command::new("/bin/bash");
                let child = child
                    .current_dir(&temp_path)
                    .env("DIP_ROOT", "lol")
                    .env("DIP_WORKDIR", &temp_path)
                    .arg("-c")
                    .arg(cmd)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
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
            Action::Exec(ref path) => {
                let mut child = Command::new(&path)
                    .current_dir(&temp_path)
                    .env("DIP_ROOT", "")
                    .env("DIP_WORKDIR", &temp_path)
                    .arg("--config")
                    .arg(config)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn_async()
                    .expect("could not spawn child");

                let stdin = child.stdin().take().unwrap();

                let input = format!("{}", input);
                let result = write_all(stdin, input)
                    .and_then(|_| child.wait_with_output())
                    .map(|output| {
                        let stdout =
                            String::from_utf8(output.stdout).unwrap_or_else(|_| String::new());
                        let stderr =
                            String::from_utf8(output.stderr).unwrap_or_else(|_| String::new());
                        json!({
                        "stdout": stdout,
                        "stderr": stderr,
                    })
                    }).map_err(|err| err_msg(format!("error: {}", err)));

                // let _result: Either<_, FutureResult<(), Error>> = {
                //     match child.stdin() {
                //         Some(ref mut stdin) => Either::A(write_all(stdin, input.as_bytes())),
                //         None => Either::B(future::err(err_msg("rip"))),
                //     }
                // };
                Box::new(result)

                // let input_s = format!("{}", input);
                // let result: Box<Future<Item = (), Error = Error> + Send> = {
                //     let rf = child.clone().lock().unwrap();
                //     match rf.stdin() {
                //         Some(ref mut stdin) => Box::new(
                //             write_all(stdin, input_s.as_bytes())
                //                 .map(|_| ())
                //                 .map_err(|err| err_msg(format!("error: {}", err))),
                //         ),
                //         None => Box::new(future::err(err_msg("Failed to acquire child stdin"))),
                //     }
                // };
                // {
                //         let rf = child.clone().lock().unwrap();
                //         let result = rf
                //             .wait_with_output()
                //             .and_then(|output| {
                //                 if output.status.success() {
                //                     future::ok(output)
                //                 } else {
                //                     // TODO: change this
                //                     future::ok(output)
                //                 }
                //             }).map(|output| {
                //                 let stdout = String::from_utf8(output.stdout)
                //                     .unwrap_or_else(|_| String::new());
                //                 let stderr = String::from_utf8(output.stderr)
                //                     .unwrap_or_else(|_| String::new());
                //                 println!("stdout: {}, stderr: {}", stdout, stderr);
                //                 json!({
                //                                 "stdout": stdout,
                //                                 "stderr": stderr,
                //                             })
                //             }).map_err(|err| err_msg(format!("could not get output: {}", err)));
                //     });

                // .and_then(move |output| {
                //     if output.status.success() {
                //         future::ok(output)
                //     } else {
                //         // TODO: get rid of unwraps
                //         future::err(err_msg(format!(
                //             "'{:?}' returned with a non-zero status code: {}\nstdout:\n{}\nstderr:\n{}",
                //             path,
                //             output.status,
                //             String::from_utf8(output.stdout).unwrap_or_else(|_| String::new()),
                //             String::from_utf8(output.stderr).unwrap_or_else(|_| String::new())
                //         )))
                //     }
                // }).map(|output| {
                //     let stdout =
                //         String::from_utf8(output.stdout).unwrap_or_else(|_| String::new());
                //     let stderr =
                //         String::from_utf8(output.stderr).unwrap_or_else(|_| String::new());
                //     // future::ok(json!({
                //     //     "stdout": stdout,
                //     //     "stderr": stderr,
                //     // }))
                //     json!("")
                // }).map_err(|err| err_msg(format!("could not get output: {}", err)))

                // let result = child
                //     .spawn_async()
                //     .expect("could not spawn child")
                //     .map_err(|err| err_msg(format!("failed to get output: {}", err)))
                //     .and_then(|child| {
                //         match child.stdin() {
                //             Some(ref mut stdin) => {
                //                 // future::result(write!(stdin, "{}", input))
                //                 future::ok(0)
                //             }
                //             None => future::err(err_msg("done fucked")),
                //         }
                //     });

                // let output = child.wait_with_output().unwrap();
                // if !output.status.success() {
                //     // TODO: get rid of unwraps
                //     return future::err(err_msg(format!(
                //         "'{:?}' returned with a non-zero status code: {}\nstdout:\n{}\nstderr:\n{}",
                //         path,
                //         output.status,
                //         String::from_utf8(output.stdout).unwrap_or_else(|_| String::new()),
                //         String::from_utf8(output.stderr).unwrap_or_else(|_| String::new())
                //     )));
                // }
                // output
            }
        };
        output.map(|x| (temp_path, x))

        // let result = match action {
        //     Action::Command(ref cmd) => {
        //         // TODO: allow some kind of simple variable replacement
        //         let child = Command::new("/bin/bash")
        //             .current_dir(&temp_path)
        //             .env("DIP_ROOT", "lol")
        //             .env("DIP_WORKDIR", &temp_path)
        //             .arg("-c")
        //             .arg(cmd)
        //             .stdin(Stdio::piped())
        //             .stdout(Stdio::piped())
        //             .stderr(Stdio::piped());
        //         child.output_async().map_err(|err| err_msg(format!("failed to get output: {}", err))).and_then(|output| {
        //             if output.status.success() {
        //                 future::ok(output)
        //             } else {
        //                 // TODO: get rid of unwraps
        //                 future::err(err_msg(format!(
        //                     "Command '{}' returned with a non-zero status code: {}\nstdout:\n{}\nstderr:\n{}",
        //                     cmd,
        //                     output.status,
        //                     String::from_utf8(output.stdout).unwrap_or_else(|_| String::new()),
        //                     String::from_utf8(output.stderr).unwrap_or_else(|_| String::new())
        //                 )))
        //             }
        //         })
        //     }
        //     Action::Exec(ref path) => {
        //         let mut child = Command::new(&path)
        //             .current_dir(&temp_path)
        //             .env("DIP_ROOT", "")
        //             .env("DIP_WORKDIR", &temp_path)
        //             .arg("--config")
        //             .arg(config)
        //             .stdin(Stdio::piped())
        //             .stdout(Stdio::piped())
        //             .stderr(Stdio::piped());
        //         child
        //             .spawn_async()
        //             .map_err(|err| err_msg(format!("failed to get output: {}", err))).and_then(|child| {
        //                 future::ok()
        //             })
        //         // {
        //         //     match child.stdin {
        //         //         Some(ref mut stdin) => {
        //         //             write!(stdin, "{}", input);
        //         //         }
        //         //         None => return future::err(err_msg("done fucked")),
        //         //     };
        //         // }
        //         // let output = child.wait_with_output().unwrap();
        //         // if !output.status.success() {
        //         //     // TODO: get rid of unwraps
        //         //     return future::err(err_msg(format!(
        //         //         "'{:?}' returned with a non-zero status code: {}\nstdout:\n{}\nstderr:\n{}",
        //         //         path,
        //         //         output.status,
        //         //         String::from_utf8(output.stdout).unwrap_or_else(|_| String::new()),
        //         //         String::from_utf8(output.stderr).unwrap_or_else(|_| String::new())
        //         //     )));
        //         // }
        //         // output
        //     }
        // };

        // let stdout = String::from_utf8(output.stdout).unwrap_or_else(|_| String::new());
        // let stderr = String::from_utf8(output.stderr).unwrap_or_else(|_| String::new());
        // future::ok((
        //     temp_path,
        //     json!({
        //     "stdout": stdout,
        //     "stderr": stderr,
        // }),
        // ))
    }
}
