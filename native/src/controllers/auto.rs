use crate::config::Config;
use crate::errors::Result;
use crate::guards::auth_token::AuthToken;
use crate::guards::authorized_controller::AuthorizedController;
use rocket::http::Status;
use rocket::{Route, State};

#[get("/mostplayed?<time_ranges>&<name>&<limit>")]
async fn mostplayed(
    token: AuthToken<'_>,
    cfg: &State<Config>,
    controller: AuthorizedController,
    time_ranges: String,
    name: Option<String>,
    limit: Option<usize>,
) -> Result<(Status, String)> {
    if let Some(auth_token) = &cfg.auth_token {
        if !matches!(token, AuthToken::Bearer(token) if token == auth_token) {
            return Ok((Status::Unauthorized, "invalid auth token".into()));
        }
    }

    let time_ranges = time_ranges.split(',').map(str::trim);
    let name = name.as_deref().unwrap_or("Current Top Songs");

    let ids = controller
        .update_mostplayed_playlists(time_ranges, name, limit)
        .await?;

    Ok((Status::Ok, format!("updated playlists: {}", ids.join(", "))))
}

#[get("/timeranges?<name>&<from>&<to>")]
async fn timeranges(
    token: AuthToken<'_>,
    cfg: &State<Config>,
    controller: AuthorizedController,
    name: Option<String>,
    from: u32,
    to: u32,
) -> Result<(Status, String)> {
    if let Some(auth_token) = &cfg.auth_token {
        if !matches!(token, AuthToken::Bearer(token) if token == auth_token) {
            return Ok((Status::Unauthorized, "invalid auth token".into()));
        }
    }

    if from >= to {
        return Ok((
            Status::BadRequest,
            "value for 'from' must be smaller than 'to'".into(),
        ));
    }

    let name = name.unwrap_or_else(|| format!("Songs from {from} to {to}"));

    let id = controller.update_timerange_playlist(from..to, name).await?;

    Ok((Status::Ok, format!("updated playlist: {id}")))
}

pub fn routes() -> Vec<Route> {
    routes![mostplayed, timeranges]
}
