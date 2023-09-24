use envconfig::{Envconfig, Error};

#[derive(Envconfig, Debug)]
pub struct Config {
    #[envconfig(from = "SA_AUTH_TOKEN")]
    pub auth_token: Option<String>,
}

impl Config {
    pub fn parse() -> Result<Self, Error> {
        Self::init_from_env()
    }
}
