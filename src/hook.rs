use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::slice::Iter;
use std::sync::Arc;

use failure::{err_msg, Error};
use futures::{future, stream, Future};
use owning_ref::BoxRef;
use serde_json::Value as JsonValue;
use tokio::{self, prelude::*};
use toml::Value;

use Handler;

pub struct Hook {
    name: String,
    handlers: Arc<Vec<Handler>>,
}

impl Hook {
    pub fn from(name: impl Into<String>, config: &Value) -> Result<Self, Error> {
        let name = name.into();
        let handlers = Arc::new(config
            .get("handlers")
            .ok_or(err_msg("No 'handlers' found."))?
            .as_array()
            .ok_or(err_msg("'handlers' is not an array."))?
            .iter()
            .map(|value: &Value| Handler::from(value))
            .collect::<Result<Vec<_>, _>>()?);
        Ok(Hook { name, handlers })
    }
    pub fn from_file<P>(path: P) -> Result<Hook, Error>
    where
        P: AsRef<Path>,
    {
        let filename = path.as_ref()
            .file_name()
            .ok_or(err_msg("what the fuck bro"))?
            .to_str()
            .ok_or(err_msg("???"))?
            .to_owned();
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config = contents.parse::<Value>()?;
        let hook = Hook::from(filename, &config)?;
        Ok(hook)
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn handle(&self, req: JsonValue, temp_path: PathBuf) -> Result<String, String> {
        let h = self.handlers.clone();
        let it = h.iter();
        let s = stream::iter_ok(it).fold((temp_path, req), move |prev, handler| {
            let (path, prev) = prev;
            handler.run(path, prev)
        });
        /*.fold(future::ok(req), |prev, handler| {
            prev.and_then(|val| handler.run(temp_path, val))
        });*/
        let s = s.map(|_| ()).map_err(|_: Error| ());
        tokio::executor::spawn(s);
        Ok("success".to_owned())
        /*
        Ok(self.iter()
            .fold(Ok(req), |prev, handler| {
                prev.and_then(|val| {
                    println!("Running {}...", handler.config());
                    let result = handler.run(&temp_path, val);
                    result
                })
            })
            .map(|res| {
                (
                    StatusCode::ACCEPTED,
                    format!(
                        "stdout:\n{}\n\nstderr:\n{}",
                        res.get("stdout").and_then(|v| v.as_str()).unwrap_or(""),
                        res.get("stderr").and_then(|v| v.as_str()).unwrap_or(""),
                    ),
                )
            })
            .unwrap_or_else(|err| (StatusCode::BAD_REQUEST, format!("Error: {:?}", err))))
            */
    }
}
