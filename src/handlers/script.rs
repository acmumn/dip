use failure::Error;
use hyper::{Body, Request, Response};

use handlers::Handler;

pub struct ScriptHandler {}

impl ScriptHandler {
    pub fn new() -> Self {
        ScriptHandler {}
    }
}

impl Handler for ScriptHandler {
    fn handle(&self, req: &Request<Body>) -> Result<Response<Body>, Error> {
        Ok(Response::new(Body::from("Lol")))
    }
}
