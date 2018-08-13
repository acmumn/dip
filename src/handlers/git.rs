use toml::Value;
use failure::Error;
use hyper::{Body, Request, Response, Method};

use handlers::Handler;

pub struct GithubHandler {}

impl GithubHandler {
    pub fn new() -> Self {
        GithubHandler {}
    }
}

impl Handler for GithubHandler {
    fn handle(&self, req: &Request<Body>) -> Result<Response<Body>, Error> {
    	if req.method() != Method::POST { bail!("Github webhooks should use the POST method.") }
        Ok(Response::new(Body::from("Lol")))
    }
}
