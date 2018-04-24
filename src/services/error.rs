use serde_json::Error as SerdeError;
use stq_http::client::Error as HttpError;

use stq_http::errors::ControllerError;

#[derive(Debug, Fail)]
pub enum ServiceError {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Http client error: {}", _0)]
    HttpClient(String),
    #[fail(display = "Parse error: {}", _0)]
    Parse(String),
    #[fail(display = "Unknown error: {}", _0)]
    Unknown(String),
}

impl From<SerdeError> for ServiceError {
    fn from(err: SerdeError) -> Self {
        ServiceError::Parse(err.to_string())
    }
}

impl From<HttpError> for ServiceError {
    fn from(err: HttpError) -> Self {
        ServiceError::HttpClient(format!("{:?}", err))
    }
}

impl From<ServiceError> for ControllerError {
    fn from(e: ServiceError) -> Self {
        match e {
            ServiceError::NotFound => ControllerError::NotFound,
            ServiceError::HttpClient(msg) => ControllerError::InternalServerError(ServiceError::HttpClient(msg).into()),
            ServiceError::Parse(msg) => ControllerError::UnprocessableEntity(ServiceError::Parse(msg).into()),
            ServiceError::Unknown(msg) => ControllerError::InternalServerError(ServiceError::Unknown(msg).into()),
        }
    }
}
