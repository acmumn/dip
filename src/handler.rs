use hyper::{Body, Request, Response};

pub trait Handler: Send + Sync {
    fn handle(&self, payload: &Request<Body>) -> Option<Response<Body>>;
}
