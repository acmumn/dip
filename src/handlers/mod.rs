mod git;
mod script;

use failure::Error;
use hyper::{Body, Request, Response};

pub use self::git::*;
pub use self::script::*;

pub trait Handler: Send + Sync {
    fn handle(&self, &Request<Body>) -> Result<Response<Body>, Error>;
}
