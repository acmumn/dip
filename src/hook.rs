use hyper::{Body, Request, Response};

pub trait Hook: Send + Sync {
    fn handle(&self, payload: &Request<Body>) -> Option<Response<Body>>;
}
