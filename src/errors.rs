use hyper::StatusCode;
use serde_json;

use stq_http::errors::{Codeable, PayloadCarrier};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Contact with this key already exists")]
    Emarsys(EmarsysError),
    #[fail(display = "Parse error")]
    Parse,
    #[fail(display = "Server is refusing to fullfil the request")]
    Forbidden,
    #[fail(display = "R2D2 connection error")]
    Connection,
    #[fail(display = "Http client error")]
    HttpClient,
}

#[derive(Debug, Serialize)]
pub struct EmarsysError {
    pub code: i64,
    pub text: Option<String>,
}

impl Codeable for Error {
    fn code(&self) -> StatusCode {
        match *self {
            Error::NotFound => StatusCode::NotFound,
            Error::Emarsys(_) => StatusCode::BadRequest,
            Error::Parse => StatusCode::UnprocessableEntity,
            Error::HttpClient | Error::Connection => StatusCode::InternalServerError,
            Error::Forbidden => StatusCode::Forbidden,
        }
    }
}

impl PayloadCarrier for Error {
    fn payload(&self) -> Option<serde_json::Value> {
        match *self {
            Error::Emarsys(ref e) => serde_json::to_value(e.clone()).ok(),
            _ => None,
        }
    }
}
