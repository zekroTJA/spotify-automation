use controller::UnauthorizedController;
use persistence::Redis;
use vercel_runtime::{http, run, Body, Error, Request, Response, StatusCode};
use vercel_utils::{expect, get_query_param};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let time_ranges = expect!(get_query_param(&req, "time_ranges")).unwrap_or("short".into());
    let name = expect!(get_query_param(&req, "name"));

    let controller = expect!(UnauthorizedController::from_env());
    let db = expect!(Redis::from_env(true));

    let refresh_token = expect!(
        expect!(db.get("spotify_automation_refresh_token")),
        http::bad_request(
            "applciation not autorized: go to /api/oauth/login to authorize the application"
        )
    );

    let controller = expect!(controller.authorize_with_token(refresh_token).await);

    let mut ids = Vec::with_capacity(3);

    for time_range in time_ranges.split(',').map(str::trim) {
        let store_key = format!("spotify_automation_playlist_id:{}", time_range);

        let playlist_id = expect!(db.get(&store_key));

        let id = expect!(
            controller
                .update_top_songs_playlist(
                    playlist_id.as_deref(),
                    &format!(
                        "{} ({} Term)",
                        name.as_deref().unwrap_or("Current Top Songs"),
                        title(time_range)
                    ),
                    Some(time_range)
                )
                .await
        );

        if playlist_id.is_none() {
            expect!(db.set(&store_key, &id.to_string()));
        }

        ids.push(id.to_string());
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::Text(format!("updated playlists: {}", ids.join(", "))))?)
}

fn title(v: &str) -> String {
    if v.is_empty() {
        return "".into();
    }
    let first = v.chars().next().unwrap().to_uppercase();
    format!("{first}{}", &v[1..])
}
