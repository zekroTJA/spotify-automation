pub mod errors;

use errors::{Error, Result};
use redis::{Client, Commands};
use std::env;

macro_rules! from_env {
    ($name:literal) => {
        env::var($name).map_err(|err| Error::EnvVar { name: $name, err })
    };
}

pub struct Redis {
    client: Client,
}

impl Redis {
    pub fn new(uri: &str) -> Result<Self> {
        let client = Client::open(uri)?;
        Ok(Redis { client })
    }

    pub fn from_env(ensure_tls: bool) -> Result<Self> {
        let mut uri = from_env!("KV_URL")?;
        // TODO: Maybe handle this with an extra env var like REDIS_ENSURE_TLS or so.
        if ensure_tls && uri.starts_with("redis://") {
            uri = format!("rediss://{}", &uri[8..])
        }
        Redis::new(&uri)
    }

    pub fn set(&self, key: &str, val: &str) -> Result<()> {
        let mut conn = self.client.get_connection()?;
        Ok(conn.set(key, val)?)
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.client.get_connection()?;
        Ok(conn.get(key)?)
    }
}
