use std::fmt::Display;

use actix_web::{http::StatusCode, ResponseError};

#[derive(Debug)]
pub enum Error {
    InvalidSignature,
    ServiceNotFound,
    Actix(actix_web::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidSignature => f.write_str("Invalid Signature"),
            Error::ServiceNotFound => f.write_str("Service not found"),
            Error::Actix(e) => e.fmt(f),
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InvalidSignature => StatusCode::FORBIDDEN,
            Error::ServiceNotFound => StatusCode::NOT_FOUND,
            Error::Actix(e) => e.as_response_error().status_code(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Actix(actix_web::Error::from(value))
    }
}

impl From<actix_web::Error> for Error {
    fn from(value: actix_web::Error) -> Self {
        Self::Actix(value)
    }
}
