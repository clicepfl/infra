use std::fmt::Display;

use actix_web::{http::StatusCode, ResponseError};

#[derive(Debug)]
pub enum Error {
    BadRequest,
    InvalidSignature,
    InvalidService,
    ForbiddenEvent,
    Actix(actix_web::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadRequest => f.write_str("Bad request"),
            Error::InvalidSignature => f.write_str("Invalid Signature"),
            Error::InvalidService => f.write_str("Invalid service"),
            Error::ForbiddenEvent => f.write_str("Event is not allowed"),
            Error::Actix(e) => e.fmt(f),
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::BadRequest => StatusCode::BAD_REQUEST,
            Error::InvalidSignature => StatusCode::FORBIDDEN,
            Error::InvalidService => StatusCode::BAD_REQUEST,
            Error::ForbiddenEvent => StatusCode::BAD_REQUEST,
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
