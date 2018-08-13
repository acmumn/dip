use failure::{err_msg, Error};
use hyper::{Body, Request, Response};
use toml::Value;

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
    pub fn handle(&self, payload: &Request<Body>) -> Option<Response<Body>> {
        None
    }
}
