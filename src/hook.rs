use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::slice::Iter;

use failure::{err_msg, Error};
use hyper::{Body, Request, Response};
use toml::Value;

use Handler;

pub struct Hook {
    name: String,
    handlers: Vec<Handler>,
}

impl Hook {
    pub fn from(name: impl Into<String>, config: &Value) -> Result<Self, Error> {
        let name = name.into();
        let handlers = config
            .get("handlers")
            .ok_or(err_msg("No 'handlers' found."))?
            .as_array()
            .ok_or(err_msg("'handlers' is not an array."))?
            .iter()
            .map(|value: &Value| Handler::from(value))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Hook { name, handlers })
    }
    pub fn from_file<P>(path: P) -> Result<Hook, Error>
    where
        P: AsRef<Path>,
    {
        let filename = path
            .as_ref()
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
    pub fn iter(&self) -> Iter<Handler> {
        self.handlers.iter()
    }
    pub fn handle(&self, _payload: &Request<Body>) -> Result<Response<Body>, Error> {
        Ok(Response::new(Body::from("lol")))
    }
}
