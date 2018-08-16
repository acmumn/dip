use std::collections::HashMap;

use futures::{future, Future, Stream};
use hyper::{Body, Error, Request, Response, StatusCode};

use {HOOKS, URIPATTERN};

pub fn dip_service(req: Request<Body>) -> Box<Future<Item = Response<Body>, Error = Error> + Send> {
    let path = req.uri().path().to_owned();
    let captures = match URIPATTERN.captures(path.as_ref()) {
        Some(value) => value,
        None => {
            return Box::new(future::ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("not found"))
                    .unwrap(),
            ))
        }
    };
    let name = match captures.name("name") {
        Some(value) => value.as_str().to_owned(),
        None => {
            return Box::new(future::ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("not found"))
                    .unwrap(),
            ))
        }
    };

    // TODO: filter by method as well

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
    let body = req.into_body();
    Box::new(body.concat2().map(move |body| {
        let req_obj = json!({
                    "body": String::from_utf8(body.to_vec()).unwrap(),
                    "headers": headers,
                    "method": method,
                });
        let hooks = HOOKS.lock().unwrap();
        let hook = hooks.get(&name).unwrap();
        let (code, msg) = hook.iter()
            .fold(Ok(req_obj), |prev, handler| {
                prev.and_then(|val| handler.run(val))
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
            .unwrap_or_else(|err| (StatusCode::BAD_REQUEST, format!("Error: {}", err)));
        Response::builder()
            .status(code)
            .body(Body::from(msg))
            .unwrap_or_else(|err| Response::new(Body::from(format!("{}", err))))
    }))
}

/* pub fn dip_service(
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
*/
