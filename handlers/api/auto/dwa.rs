use controller::UnauthorizedController;
use persistence::redis::Redis;
use vercel_runtime::{http, run, Body, Error, Request, Response, StatusCode};
use vercel_utils::{expect, get_query_param};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let dw_name = expect!(get_query_param(&req, "dw_name"));
    let dwa_name = expect!(get_query_param(&req, "dwa_name"));

    let db = expect!(Redis::from_env(true));
    let controller = expect!(UnauthorizedController::from_env(db));

    let controller = expect!(controller.authorize_from_db().await,
        Err(err) if matches!(err, controller::errors::Error::NoAuthToken) => http::bad_request("no authorization token stored"),
        Err(err) => http::internal_server_error(err.to_string()));

    let dw_name = dw_name.as_deref().unwrap_or("Discover Weekly");
    let id = expect!(
        controller
            .archive_discover_weekly(dw_name, dwa_name.unwrap_or(format!("{dw_name} Archive")))
            .await
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::Text(format!("updated playlist: {id}")))?)
}
