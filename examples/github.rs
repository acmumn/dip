extern crate dip;
extern crate hmac;
extern crate secstr;
extern crate serde_json;
extern crate sha1;
#[macro_use]
extern crate serde_derive;
extern crate failure;
extern crate generic_array;
#[macro_use]
extern crate structopt;

use std::collections::HashMap;
use std::io::{self, Read};
use std::iter::FromIterator;
use std::path::PathBuf;

use failure::{err_msg, Error};
use generic_array::GenericArray;
use hmac::{Hmac, Mac};
use secstr::*;
use sha1::Sha1;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    /// JSON input
    #[structopt(short = "c", long = "config")]
    pub config: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    secret: String,
    outdir: PathBuf,
    disable_hmac_verify: bool,
}

#[derive(Serialize, Deserialize)]
struct Payload {
    body: String,
    headers: HashMap<String, String>,
}

fn main() -> Result<(), Error> {
    let args = Opt::from_args();
    let config: Config = serde_json::from_str(&args.config)?;
    println!("{:?}", config);

    let mut payload = String::new();
    io::stdin().read_to_string(&mut payload)?;
    println!("raw payload: {}", payload);
    let payload: Payload = serde_json::from_str(&payload)?;
    println!("processed payload: {}", payload.body);

    if !config.disable_hmac_verify {
        let secret = GenericArray::from_iter(config.secret.bytes());
        let mut mac = Hmac::<Sha1>::new(&secret);
        mac.input(payload.body.as_bytes());
        let signature = mac.result()
            .code()
            .into_iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join("");

        let auth = payload
            .headers
            .get("x-hub-signature")
            .ok_or(err_msg("Missing auth header"))?;

        let left = SecStr::from(format!("sha1={}", signature));
        let right = SecStr::from(auth.bytes().collect::<Vec<_>>());
        assert!(left == right, "HMAC signature didn't match");
    }

    println!("gonna clone it to {:?}", config.outdir);
    Ok(())
}
