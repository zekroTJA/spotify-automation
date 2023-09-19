use controller::UnauthorizedController;
use persistence::redis::Redis;
use vercel_runtime::{http, run, Body, Error, Request, Response, StatusCode};
use vercel_utils::{expect, get_query_param, get_query_param_parsed};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let name = expect!(get_query_param(&req, "name"));
    let from: Option<u32> = expect!(get_query_param_parsed(&req, "from"), Err(err) => http::bad_request(format!("invalid 'from' value: {err}")));
    let to: Option<u32> = expect!(get_query_param_parsed(&req, "to"), Err(err) => http::bad_request(format!("invalid 'to' value: {err}")));

    let Some(from) = from else {
        return http::bad_request("'from' value must be given");
    };

    let Some(to) = to else {
        return http::bad_request("'to' value must be given");
    };

    if from >= to {
        return http::bad_request("value for 'from' must be smaller than 'to'");
    }

    let db = expect!(Redis::from_env(true));
    let controller = expect!(UnauthorizedController::from_env(db));

    let controller = expect!(controller.authorize_from_db().await,
        Err(err) if matches!(err, controller::errors::Error::NoAuthToken) => http::bad_request("no authorization token stored"),
        Err(err) => http::internal_server_error(err.to_string()));

    let name = name.unwrap_or_else(|| format!("Songs from {} to {}", from, to));
    let id = expect!(controller.update_timerange_playlist(from..to, name).await);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::Text(format!("updated playlist: {id}")))?)
}
