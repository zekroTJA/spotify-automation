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

    pub fn from_env() -> Result<Self> {
        let uri = from_env!("KV_URL")?;
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
