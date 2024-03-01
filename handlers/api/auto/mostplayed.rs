use controller::UnauthorizedController;
use persistence::redis::Redis;
use vercel_runtime::{http, run, Body, Error, Request, Response, StatusCode};
use vercel_utils::{expect, get_query_param, get_query_param_parsed};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let time_ranges = expect!(get_query_param(&req, "time_ranges")).unwrap_or("short".into());
    let name = expect!(get_query_param(&req, "name"));
    let limit: Option<usize> = expect!(get_query_param_parsed(&req, "limit"));

    let db = expect!(Redis::from_env(true));
    let controller = expect!(UnauthorizedController::from_env(db));

    let controller = expect!(controller.authorize_from_db().await,
        Err(err) if matches!(err, controller::errors::Error::NoAuthToken) => http::bad_request("no authorization token stored"),
        Err(err) => http::internal_server_error(err.to_string()));

    let time_ranges = time_ranges.split(',').map(str::trim);
    let ids = expect!(
        controller
            .update_mostplayed_playlists(
                time_ranges,
                name.as_deref().unwrap_or("Current Top Songs"),
                limit,
            )
            .await
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::Text(format!("updated playlists: {}", ids.join(", "))))?)
}
