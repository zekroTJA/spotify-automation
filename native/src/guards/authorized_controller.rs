use controller::UnauthorizedController;
use persistence::redis::Redis;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};
use std::ops::Deref;

pub struct AuthorizedController(controller::AuthorizedController<Redis>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthorizedController {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let res = request
            .guard::<&State<UnauthorizedController<Redis>>>()
            .await;

        let controller = match res {
            Outcome::Success(v) => v,
            Outcome::Failure(err) => return Outcome::Failure(err),
            Outcome::Forward(fw) => return Outcome::Forward(fw),
        };

        let Ok(authorized_controller) = controller.authorize_from_db().await else {
            return Outcome::Failure((Status::Unauthorized, ()));
        };

        Outcome::Success(Self(authorized_controller))
    }
}

impl Deref for AuthorizedController {
    type Target = controller::AuthorizedController<Redis>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
