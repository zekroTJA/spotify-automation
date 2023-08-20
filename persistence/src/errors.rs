use std::env::VarError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("env variable not found: {name}: {err}")]
    EnvVar { name: &'static str, err: VarError },
}
