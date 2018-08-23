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
use std::env;
use std::io::{self, Read};
use std::iter::FromIterator;
use std::path::PathBuf;
use std::process::Command;

use failure::err_msg;
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
    #[serde(default)]
    disable_hmac_verify: bool,
    #[serde(default = "default_path")]
    path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct Payload {
    body: String,
    headers: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RepositoryInfo {
    clone_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GithubPayload {
    repository: RepositoryInfo,
}

fn default_path() -> PathBuf {
    PathBuf::from(".")
}

fn main() {
    let args = Opt::from_args();
    let config: Config = serde_json::from_str(&args.config).expect("Could not parse config.");

    let mut payload = String::new();
    io::stdin()
        .read_to_string(&mut payload)
        .expect("Could not read from stdin");
    let payload: Payload = serde_json::from_str(&payload)
        .expect(&format!("Could not parse stdin into json: '{}'", payload));

    if !config.disable_hmac_verify {
        let secret = GenericArray::from_iter(config.secret.bytes());
        let mut mac = Hmac::<Sha1>::new(&secret);
        mac.input(payload.body.as_bytes());
        let signature = mac
            .result()
            .code()
            .into_iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join("");

        let auth = payload
            .headers
            .get("x-hub-signature")
            .ok_or(err_msg("Missing auth header"))
            .expect("Missing auth header");

        let left = SecStr::from(format!("sha1={}", signature));
        let right = SecStr::from(auth.bytes().collect::<Vec<_>>());
        assert!(
            left == right,
            "HMAC signature didn't match: {} vs. {}",
            signature,
            auth
        );
    }

    let payload: GithubPayload =
        serde_json::from_str(&payload.body).expect("Could not parse Github input into json");
    let mut target_path =
        PathBuf::from(env::var("DIP_WORKDIR").expect("Could not determine working directory"));
    target_path.push(&config.path);
    Command::new("git")
        .arg("clone")
        .arg(&payload.repository.clone_url)
        .arg("--recursive")
        .arg("--depth")
        .arg("1")
        .arg(&target_path)
        .output()
        .expect("Could not spawn process to clone");
}
