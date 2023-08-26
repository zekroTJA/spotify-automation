use std::env::VarError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("spotify client error: {0}")]
    SpotifyClient(#[from] rspotify::ClientError),

    #[error("mutex lock is poisoned")]
    LockPoisoned,

    #[error("no auth token obtainable; this is a bug - yikes")]
    NoAuthToken,

    #[error("env variable not found: {name}: {err}")]
    EnvVar { name: &'static str, err: VarError },

    #[error("spotify id error: {0}")]
    SpotifyId(#[from] rspotify::model::IdError),

    #[error("database error: {0}")]
    Database(#[from] persistence::errors::Error),

    #[error("invalid time range")]
    InvalidTimeRange,

    #[error("no authorization token has been stored before")]
    NoTokenStored,

    #[error("no playlist found")]
    NoPlaylistFound,
}
