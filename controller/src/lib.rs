pub mod errors;

use self::errors::Error;
use errors::Result;
use futures::stream::TryStreamExt;
use persistence::KV;
use rspotify::{
    model::{FullPlaylist, FullTrack, PlaylistId, TimeRange},
    prelude::{BaseClient, OAuthClient, PlayableId},
    scopes, AuthCodeSpotify, Config, Credentials, OAuth, Token,
};
use std::env;

const DBKEY_REFRESH_TOKEN: &str = "spotify_automation_refresh_token";
const DBKEY_PLAYLIST_MOSTPLAYED_PREFIX: &str = "spotify_automation_playlist_id";

macro_rules! from_env {
    ($name:literal) => {
        env::var($name).map_err(|err| Error::EnvVar { name: $name, err })
    };
}

pub struct UnauthorizedController<DB: KV> {
    client: AuthCodeSpotify,
    db: DB,
}

pub struct AuthorizedController<DB: KV> {
    client: AuthCodeSpotify,
    db: DB,
}

impl<DB: KV> UnauthorizedController<DB> {
    pub fn new(
        client_id: &str,
        client_secret: &str,
        redirect_uri: String,
        db: DB,
    ) -> UnauthorizedController<DB> {
        let config = Config {
            ..Default::default()
        };

        let oauth = OAuth {
            redirect_uri,
            scopes: scopes!(
                "user-top-read",
                "playlist-modify-public",
                "playlist-modify-private"
            ),
            ..Default::default()
        };

        let creds = Credentials::new(client_id, client_secret);
        let client = AuthCodeSpotify::with_config(creds, oauth, config);

        UnauthorizedController { client, db }
    }

    pub fn from_env(db: DB) -> Result<UnauthorizedController<DB>> {
        let client_id = from_env!("SPOTIFY_CLIENTID")?;
        let client_secret = from_env!("SPOTIFY_CLIENTSECRET")?;
        let redirect_uri = from_env!("REDIRECT_URL")?;
        Ok(UnauthorizedController::new(
            &client_id,
            &client_secret,
            redirect_uri,
            db,
        ))
    }

    pub fn get_authorize_url(&self) -> Result<String> {
        Ok(self.client.get_authorize_url(true)?)
    }

    pub async fn authorize_with_code(self, code: &str) -> Result<AuthorizedController<DB>> {
        self.client.request_token(code).await?;

        Ok(AuthorizedController {
            client: self.client,
            db: self.db,
        })
    }

    pub async fn authorize_with_token(self, token: String) -> Result<AuthorizedController<DB>> {
        let token = Token {
            refresh_token: Some(token),
            ..Default::default()
        };

        *(self
            .client
            .token
            .lock()
            .await
            .map_err(|_| Error::LockPoisoned)?) = Some(token);

        self.client.refresh_token().await?;

        Ok(AuthorizedController {
            client: self.client,
            db: self.db,
        })
    }

    pub async fn authorize_from_db(self) -> Result<AuthorizedController<DB>> {
        let Some(token) = self.db.get(DBKEY_REFRESH_TOKEN)? else {
            return Err(Error::NoAuthToken);
        };

        self.authorize_with_token(token).await
    }
}

impl<DB: KV> AuthorizedController<DB> {
    pub async fn refresh_token(&self) -> Result<String> {
        let token = self.client.get_token();
        let token = token.lock().await.map_err(|_| Error::LockPoisoned)?;
        let token = token.to_owned().ok_or(Error::NoAuthToken)?;

        token.refresh_token.ok_or(Error::NoAuthToken)
    }

    pub async fn store_token(&self) -> Result<()> {
        let token = self.refresh_token().await?;
        self.db.set(DBKEY_REFRESH_TOKEN, token)?;
        Ok(())
    }

    pub async fn get_top_songs(&self, time_range: Option<TimeRange>) -> Result<Vec<FullTrack>> {
        let top_tracks = self.client.current_user_top_tracks(time_range);

        let tracks: std::result::Result<Vec<_>, _> = top_tracks.try_collect().await;

        Ok(tracks?)
    }

    pub async fn create_playlist(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> Result<FullPlaylist> {
        let me = self.client.current_user().await?;
        let playlist = self
            .client
            .user_playlist_create(me.id, name, Some(false), Some(false), description)
            .await?;
        Ok(playlist)
    }

    pub async fn update_playlist(
        &self,
        id: PlaylistId<'_>,
        items: Vec<PlayableId<'_>>,
    ) -> Result<()> {
        let current_items = self.client.playlist_items(id.clone(), None, None);
        let current_items: std::result::Result<Vec<_>, _> = current_items.try_collect().await;
        let current_items = current_items?;

        let current_item_ids: Vec<_> = current_items
            .iter()
            .filter_map(|i| i.track.as_ref())
            .filter_map(|i| i.id())
            .collect();

        self.client
            .playlist_remove_all_occurrences_of_items(id.clone(), current_item_ids, None)
            .await?;

        self.client.playlist_add_items(id, items, None).await?;

        Ok(())
    }

    pub async fn update_top_songs_playlist<'a, T: AsRef<str>>(
        &'a self,
        id: Option<&'a str>,
        name: &str,
        time_range: Option<T>,
    ) -> Result<PlaylistId> {
        let time_range = time_range.map(time_range_from_str).transpose()?;

        let playlist_id: PlaylistId<'a> = match id {
            Some(id) => PlaylistId::from_id_or_uri(id)?,
            None => self.create_playlist(name, None).await?.id,
        };

        let top_songs = self
            .get_top_songs(time_range)
            .await?
            .iter()
            .cloned()
            .filter_map(|i| i.id)
            .map(|v| v.into())
            .collect();

        self.update_playlist(playlist_id.clone(), top_songs).await?;

        Ok(playlist_id)
    }

    pub async fn update_mostplayed_playlists<I, E, N>(
        &self,
        time_ranges: I,
        name_prefix: N,
    ) -> Result<Vec<String>>
    where
        I: Iterator<Item = E>,
        E: AsRef<str>,
        N: AsRef<str>,
    {
        let mut ids = Vec::with_capacity(3);

        for time_range in time_ranges {
            let time_range = time_range.as_ref();
            let store_key = format!("{}:{}", DBKEY_PLAYLIST_MOSTPLAYED_PREFIX, time_range);
            let playlist_id = self.db.get(store_key)?;
            let playlist_name = format!("{} ({} Term)", name_prefix.as_ref(), title(time_range));

            let id = self
                .update_top_songs_playlist(playlist_id.as_deref(), &playlist_name, Some(time_range))
                .await?;

            ids.push(id.to_string());
        }

        Ok(ids)
    }
}

fn time_range_from_str<T: AsRef<str>>(v: T) -> Result<TimeRange> {
    match v.as_ref() {
        "long" => Ok(TimeRange::LongTerm),
        "medium" => Ok(TimeRange::MediumTerm),
        "short" => Ok(TimeRange::ShortTerm),
        _ => Err(Error::InvalidTimeRange),
    }
}

fn title(v: &str) -> String {
    if v.is_empty() {
        return "".into();
    }
    let first = v.chars().next().unwrap().to_uppercase();
    format!("{first}{}", &v[1..])
}
