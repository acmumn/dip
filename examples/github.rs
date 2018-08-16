extern crate dip;
extern crate secstr;
extern crate serde_json;
extern crate hmac;
extern crate sha1;
#[macro_use]
extern crate serde_derive;
extern crate generic_array;
extern crate failure;
#[macro_use]
extern crate structopt;

use std::io::{self, Read};
use std::collections::HashMap;

use secstr::*;
use hmac::{Mac,Hmac};
use sha1::Sha1;
use generic_array::GenericArray;
use failure::{err_msg, Error};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    /// JSON input
    #[structopt(short = "c", long = "config")]
    pub config: String,
}

#[derive(Serialize, Deserialize)]
struct Config {
    secret: String,
}

#[derive(Serialize, Deserialize)]
struct Payload {
    body: String,
    headers: HashMap<String, String>,
}

fn main() -> Result<(), Error> {
    let args = Opt::from_args();
    let config: Config = serde_json::from_str(&args.config)?;

    let mut payload = String::new();
    io::stdin().read_to_string(&mut payload)?;
    let payload: Payload = serde_json::from_str(&payload)?;

    let secret = GenericArray::from_slice(config.secret.as_bytes());
    let mut mac = Hmac::<Sha1>::new(secret);
    mac.input(payload.body.as_bytes());
    let signature = mac.result().code().into_iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join("");
    
    let auth = payload.headers.get("X-Hub-Signature").ok_or(err_msg("Missing auth header."))?;

    let left = SecStr::from(format!("sha1={}", signature));
    let right = SecStr::from(auth.bytes().collect::<Vec<_>>());
    assert_eq!(left, right);

    println!("{}", payload.body);
    Ok(())
}
