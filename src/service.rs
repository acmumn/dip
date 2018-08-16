use std::collections::HashMap;
use std::thread;

use futures::{future, Future, Stream};
use hyper::{Body, Error, Request, Response, StatusCode};
use mktemp::Temp;

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

    // spawn job
    thread::spawn(move || {
        req.into_body().concat2().map(move |body| {
            let body = String::from_utf8(body.to_vec()).unwrap();
            let req_obj = json!({
                "body": body, 
                "headers": headers,
                "method": method,
            });
            let hooks = HOOKS.lock().unwrap();
            {
                let mut temp_dir = Temp::new_dir().unwrap();
                let temp_path = temp_dir.to_path_buf();
                assert!(temp_path.exists());

                let hook = hooks.get(&name).unwrap();
                let (code, msg) = hook.iter()
                    .fold(Ok(req_obj), |prev, handler| {
                        prev.and_then(|val| {
                            println!("Running {}...", handler.config());
                            let result = handler.run(&temp_path, val);
                            println!("{:?}", result);
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
                    .unwrap_or_else(|err| (StatusCode::BAD_REQUEST, format!("Error: {:?}", err)));

                temp_dir.release();
                Response::builder()
                    .status(code)
                    .body(Body::from(msg))
                    .unwrap_or_else(|err| Response::new(Body::from(format!("{}", err))))
            }
        })
    });

    Box::new(future::ok(
        Response::builder()
            .status(StatusCode::ACCEPTED)
            // TODO: assign job a uuid and do some logging
            .body(Body::from(format!("job {} started", -1)))
            .unwrap(),
    ))
}
