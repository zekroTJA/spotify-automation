use crate::errors::Result;
use controller::UnauthorizedController;
use persistence::redis::Redis;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::{Route, State};

#[get("/login")]
async fn login(controller: &State<UnauthorizedController<Redis>>) -> Result<Redirect> {
    let auth_url = controller.get_authorize_url()?;
    Ok(Redirect::temporary(auth_url))
}

#[get("/callback?<code>")]
async fn callback(
    controller: &State<UnauthorizedController<Redis>>,
    code: String,
) -> Result<(Status, &'static str)> {
    let controller = controller.authorize_with_code(&code).await?;
    controller.store_token().await?;
    Ok((Status::Ok, "authorized"))
}

pub fn routes() -> Vec<Route> {
    routes![login, callback]
}
