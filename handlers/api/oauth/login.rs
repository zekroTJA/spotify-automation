use controller::UnauthorizedController;
use persistence::noop::NoOp;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use vercel_utils::expect;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    let controller = expect!(UnauthorizedController::from_env(NoOp));
    let auth_url = expect!(controller.get_authorize_url());

    Ok(Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header("Location", auth_url)
        .body(Body::Empty)?)
}
