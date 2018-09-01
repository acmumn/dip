use std::collections::HashMap;
use std::env;
use std::iter::FromIterator;
use std::path::PathBuf;
use std::process::Command;

use failure::{err_msg, Error};
use generic_array::GenericArray;
use hmac::{Hmac, Mac};
use secstr::*;
use serde::Serialize;
use serde_json::{self, Serializer as JsonSerializer, Value as JsonValue};
use sha1::Sha1;
use structopt::StructOpt;
use toml::Value as TomlValue;

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

pub fn main(config: &TomlValue, input: &JsonValue) -> Result<JsonValue, Error> {
    let config_str = {
        let mut buf: Vec<u8> = Vec::new();
        {
            let mut serializer = JsonSerializer::new(&mut buf);
            TomlValue::serialize(&config, &mut serializer).unwrap();
        }
        String::from_utf8(buf).unwrap()
    };
    let config: Config = serde_json::from_str(&config_str)?;

    let payload_str = format!("{}", input);
    let payload: Payload = serde_json::from_str(&payload_str)?;

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
    Ok(json!(1))
}
