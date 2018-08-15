mod builder;

use failure::Error;
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

pub use self::builder::*;

pub struct Handler {
    pub handle: fn(Result<JsonValue, Error>) -> Result<JsonValue, Error>,
}

impl Handler {
    pub fn from(config: &TomlValue) -> Result<Self, Error> {
        bail!("rip")
    }
}
