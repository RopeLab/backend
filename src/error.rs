use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use axum::response::{IntoResponse, Response};
use http::StatusCode;

pub type Result<T> = std::result::Result<T, APIError>;

#[derive(Debug, Clone)]
#[derive(axum_enum_response::EnumIntoResponse)]
pub enum APIError {
    #[status_code(INTERNAL_SERVER_ERROR)]
    Internal(#[key("error")] String),

    #[status_code(UNAUTHORIZED)]
    UNAUTHORIZED,

    #[status_code(FORBIDDEN)]
    #[message("EmailUsed")]
    EmailUsed,

    #[status_code(FORBIDDEN)]
    #[message("InvalidCredentials")]
    InvalidCredentials,
}


impl APIError {
    pub fn internal<E: Display>(error: E) -> Self {
        APIError::Internal(format!("{:}", error))
    }
}

impl Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for APIError {

}

