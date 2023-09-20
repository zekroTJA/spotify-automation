use controller::errors::Error as ControllerError;
use rocket::http::Status;

#[derive(Responder)]
pub struct ErrorResponse((Status, String));

pub type Result<T> = core::result::Result<T, ErrorResponse>;

impl From<controller::errors::Error> for ErrorResponse {
    fn from(err: ControllerError) -> Self {
        let status = match err {
            ControllerError::InvalidTimeRange
            | ControllerError::InvalidYear(_)
            | ControllerError::NoAuthToken => Status::BadRequest,
            ControllerError::AuthorizationFailed(_) => Status::Unauthorized,
            _ => Status::InternalServerError,
        };

        Self((status, err.to_string()))
    }
}
