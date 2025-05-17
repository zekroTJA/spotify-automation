use rocket::request::{FromRequest, Outcome};
use rocket::Request;

#[allow(dead_code)]
pub enum AuthToken<'r> {
    None,
    Basic(&'r str),
    Bearer(&'r str),
    Uncategorized((&'r str, &'r str)),
    Unprefixed(&'r str),
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthToken<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Some(header) = request.headers().get_one("Authorization") else {
            return Outcome::Success(Self::None);
        };

        let Some((prefix, value)) = header.split_once(' ') else {
            return Outcome::Success(Self::Unprefixed(header));
        };

        Outcome::Success(match prefix.to_lowercase().as_str() {
            "basic" => Self::Basic(value),
            "bearer" => Self::Bearer(value),
            _ => Self::Uncategorized((prefix, value)),
        })
    }
}
