pub mod errors;

use self::errors::Error;
use errors::Result;
use futures::stream::TryStreamExt;
use futures::{future, StreamExt};
use persistence::KV;
use rspotify::model::{FullPlaylist, FullTrack, PlaylistId, SimplifiedPlaylist, TimeRange};
use rspotify::prelude::{BaseClient, OAuthClient, PlayableId};
use rspotify::{scopes, AuthCodeSpotify, Config, Credentials, OAuth, Token};
use std::env;
use std::ops::Range;
use std::sync::Arc;

const DBKEY_REFRESH_TOKEN: &str = "spotify_automation_refresh_token";
const DBKEY_PLAYLIST_MOSTPLAYED_PREFIX: &str = "spotify_automation_playlist_id";
const DBKEY_PLAYLIST_TIMERANGE_PREFIX: &str = "spotify_automation_timerange_id";

macro_rules! from_env {
    ($name:literal) => {
        env::var($name).map_err(|err| Error::EnvVar { name: $name, err })
    };
}

pub struct UnauthorizedController<DB: KV> {
    client: AuthCodeSpotify,
    db: Arc<DB>,
}

pub struct AuthorizedController<DB: KV> {
    client: AuthCodeSpotify,
    db: Arc<DB>,
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
                "user-library-read",
                "playlist-read-private",
                "playlist-modify-public",
                "playlist-modify-private"
            ),
            ..Default::default()
        };

        let creds = Credentials::new(client_id, client_secret);
        let client = AuthCodeSpotify::with_config(creds, oauth, config);

        UnauthorizedController {
            client,
            db: Arc::new(db),
        }
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

    pub async fn authorize_with_code(&self, code: &str) -> Result<AuthorizedController<DB>> {
        self.client
            .request_token(code)
            .await
            .map_err(|err| Error::AuthorizationFailed(err.into()))?;

        Ok(AuthorizedController {
            client: self.client.clone(),
            db: self.db.clone(),
        })
    }

    pub async fn authorize_with_token(&self, token: String) -> Result<AuthorizedController<DB>> {
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
            client: self.client.clone(),
            db: self.db.clone(),
        })
    }

    pub async fn authorize_from_db(&self) -> Result<AuthorizedController<DB>> {
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

    pub async fn get_top_songs(
        &self,
        time_range: Option<TimeRange>,
        limit: Option<usize>,
    ) -> Result<Vec<FullTrack>> {
        let top_tracks = self.client.current_user_top_tracks(time_range);

        let tracks: std::result::Result<Vec<_>, _> =
            top_tracks.take(limit.unwrap_or(100)).try_collect().await;

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

        for chunks in current_item_ids.chunks(100) {
            let chunks = chunks.iter().map(|id| id.clone_static());
            self.client
                .playlist_remove_all_occurrences_of_items(id.clone_static(), chunks, None)
                .await?;
        }

        for chunks in items.chunks(100) {
            let chunks = chunks.iter().map(|id| id.clone_static());
            self.client
                .playlist_add_items(id.clone_static(), chunks, None)
                .await?;
        }

        Ok(())
    }

    pub async fn update_top_songs_playlist<'a, T: AsRef<str>>(
        &'a self,
        id: Option<&'a str>,
        name: &str,
        time_range: Option<T>,
        limit: Option<usize>,
    ) -> Result<PlaylistId<'a>> {
        let time_range = time_range.map(time_range_from_str).transpose()?;

        let playlist_id: PlaylistId<'a> = match id {
            Some(id) => PlaylistId::from_id_or_uri(id)?,
            None => self.create_playlist(name, None).await?.id,
        };

        let top_songs = self
            .get_top_songs(time_range, limit)
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
        limit: Option<usize>,
    ) -> Result<Vec<String>>
    where
        I: Iterator<Item = E>,
        E: AsRef<str>,
        N: AsRef<str>,
    {
        let mut ids = Vec::with_capacity(3);

        for time_range in time_ranges {
            let time_range = time_range.as_ref();
            let store_key = format!("{DBKEY_PLAYLIST_MOSTPLAYED_PREFIX}:{time_range}");
            let playlist_id = self.db.get(store_key)?;
            let playlist_name = format!("{} ({} Term)", name_prefix.as_ref(), title(time_range));

            let id = self
                .update_top_songs_playlist(
                    playlist_id.as_deref(),
                    &playlist_name,
                    Some(time_range),
                    limit,
                )
                .await?;

            ids.push(id.to_string());
        }

        Ok(ids)
    }

    pub async fn find_playlist<P>(&self, preticate: P) -> Result<SimplifiedPlaylist>
    where
        P: FnMut(&&SimplifiedPlaylist) -> bool + Copy,
    {
        let mut offset = 0;
        const PAGE_SIZE: u32 = 50;

        loop {
            let playlists = self
                .client
                .current_user_playlists_manual(Some(PAGE_SIZE), Some(offset))
                .await?;

            if playlists.items.is_empty() {
                break;
            }

            if let Some(playlist) = playlists.items.iter().find(preticate) {
                return Ok(playlist.clone());
            }

            offset += PAGE_SIZE;
        }

        Err(Error::NoPlaylistFound)
    }

    pub async fn update_timerange_playlist(
        &self,
        year_range: Range<u32>,
        playlist_name: impl AsRef<str>,
    ) -> Result<PlaylistId> {
        let iter = self.client.current_user_saved_tracks(None).filter(|t| {
            future::ready(t.as_ref().is_ok_and(|t| {
                t.track
                    .album
                    .release_date
                    .as_ref()
                    .is_some_and(|date| year(date).is_ok_and(|year| year_range.contains(&year)))
            }))
        });

        let items: std::result::Result<Vec<_>, _> = iter.try_collect().await;

        let item_ids = items?
            .iter()
            .cloned()
            .map(|p| p.track)
            .filter_map(|t| t.id.as_ref().map(|id| id.clone_static()))
            .map(PlayableId::from)
            .collect();

        let store_key = format!(
            "{}:{}-{}",
            DBKEY_PLAYLIST_TIMERANGE_PREFIX, year_range.start, year_range.end
        );

        let playlist_id = self.db.get(&store_key)?;
        let playlist_id = match playlist_id.as_deref() {
            Some(id) => PlaylistId::from_id_or_uri(id)?,
            None => {
                let id = self.create_playlist(playlist_name.as_ref(), None).await?.id;
                self.db.set(store_key, id.to_string())?;
                id
            }
        };

        self.update_playlist(playlist_id.clone(), item_ids).await?;

        Ok(playlist_id.clone_static())
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

fn year(v: &str) -> Result<u32> {
    let mut year = v;

    if let Some(idx) = v.chars().position(|c| c == '-') {
        year = &year[..idx];
    }

    Ok(year.parse()?)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_year() {
        assert!(matches!(year("2023"), Ok(v) if v == 2023));
        assert!(matches!(year("1998-12-12"), Ok(v) if v == 1998));
    }
}
