use controller::UnauthorizedController;
use persistence::Redis;
use vercel_runtime::{http, run, Body, Error, Request, Response, StatusCode};
use vercel_utils::{expect, get_query_param};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let controller = expect!(UnauthorizedController::from_env());
    let db = expect!(Redis::from_env());

    let code = expect!(
        expect!(get_query_param(&req, "code")),
        http::bad_request("invalid authorization code")
    );

    let controller = expect!(controller.authorize_with_code(&code).await);
    let refresh_token = expect!(controller.refresh_token().await);

    expect!(db.set("spotify_automation_refresh_token", &refresh_token));

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::Text("authorized".into()))?)
}
