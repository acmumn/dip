use std::collections::HashMap;

use failure::{err_msg, Error};
use futures::{future, Future};
use hyper::{Body, Request, Response, StatusCode};

use {HOOKS, NOTFOUND, URIPATTERN};

fn service_fn(req: Request<Body>) -> Result<Response<Body>, Error> {
    let path = req.uri().path().to_owned();
    let captures = URIPATTERN
        .captures(path.as_ref())
        .ok_or(err_msg("Did not match url pattern"))?;
    let name = captures
        .name("name")
        .ok_or(err_msg("Missing name"))?
        .as_str();
    let hooks = HOOKS.lock().unwrap();
    let hook = hooks
        .get(name)
        .ok_or(err_msg(format!("Hook '{}' doesn't exist", name)))?;

    let req_obj = {
        let headers = req.headers()
            .clone()
            .into_iter()
            .filter_map(|(k, v)| {
                let key = k.unwrap().as_str().to_owned();
                v.to_str().map(|value| (key, value.to_owned())).ok()
            })
            .collect::<HashMap<_, _>>();
        let method = req.method().as_str().to_owned();
        // probably not idiomatically the best way to do it
        // i was just trying to get something working
        let body = "wip".to_owned();
        json!({
            "body": body,
            "headers": headers,
            "method": method,
        })
    };
    hook.iter()
        .fold(Ok(req_obj), |prev, handler| {
            prev.and_then(|val| handler.run(val))
        })
        .map(|_| Response::new(Body::from("success")))
}

pub fn dip_service(
    req: Request<Body>,
) -> Box<Future<Item = Response<Body>, Error = String> + Send> {
    let uri = req.uri().path().to_owned();
    Box::new(future::ok(service_fn(req).unwrap_or_else(|err| {
        eprintln!("Error from '{}': {}", uri, err);
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(NOTFOUND))
            .unwrap()
    })))
}
