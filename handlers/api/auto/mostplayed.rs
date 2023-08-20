use controller::UnauthorizedController;
use persistence::Redis;
use vercel_runtime::{http, run, Body, Error, Request, Response, StatusCode};
use vercel_utils::expect;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    let controller = expect!(UnauthorizedController::from_env());
    let db = expect!(Redis::from_env());

    let refresh_token = expect!(
        expect!(db.get("spotify_automation_refresh_token")),
        http::bad_request(
            "applciation not autorized: go to /api/oauth/login to authorize the application"
        )
    );

    let controller = expect!(controller.authorize_with_token(refresh_token).await);

    let playlist_id = expect!(db.get("spotify_automation_playlist_id"));

    let id = expect!(
        controller
            .update_top_songs_playlist(playlist_id.as_deref(), "Current Top Songs")
            .await
    );

    if playlist_id.is_none() {
        expect!(db.set("spotify_automation_playlist_id", &id.to_string()));
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::Empty)?)
}
