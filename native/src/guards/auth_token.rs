use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};

pub enum AuthToken {
    Basic(String),
    Bearer(String),
    Uncategorized((String, String)),
    Unprefixed(String),
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthToken {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Some(header) = request.headers().get_one("Authorization") else {
            return Outcome::Failure((Status::Unauthorized, ()));
        };

        todo!()
    }
}
