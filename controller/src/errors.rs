use std::env::VarError;
use std::num::ParseIntError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("spotify client error: {0}")]
    SpotifyClient(#[from] rspotify::ClientError),

    #[error("spotify authorization failed: {0}")]
    AuthorizationFailed(Box<dyn std::error::Error + Send + Sync>),

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

    #[error("playlist by id doesn ot exist")]
    PlaylistDoesNotExist,

    #[error("invalid year: {0}")]
    InvalidYear(#[from] ParseIntError),
}
