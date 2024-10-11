use std::error::Error;
use std::fmt;
use std::fmt::{Display};

pub type APIResult<T> = Result<T, APIError>;

#[derive(Debug, Clone)]
#[derive(axum_enum_response::EnumIntoResponse)]
pub enum APIError {
    #[status_code(INTERNAL_SERVER_ERROR)]
    Internal(#[key("error")] String),

    #[status_code(UNAUTHORIZED)]
    UNAUTHORIZED,

    #[status_code(FORBIDDEN)]
    #[message("Email used")]
    EmailUsed,

    #[status_code(FORBIDDEN)]
    #[message("Invalid credentials")]
    InvalidCredentials,

    #[status_code(NOT_ACCEPTABLE)]
    #[message("Permission already added")]
    PermissionAlreadyAdded,

    #[status_code(NOT_ACCEPTABLE)]
    #[message("Permission not there")]
    PermissionNotThere,

    #[status_code(NOT_ACCEPTABLE)]
    #[message("Invalid path")]
    InvalidPath,

    #[status_code(NOT_ACCEPTABLE)]
    #[message("Event ids dont match")]
    EventIdsDontMatch,

    #[status_code(NOT_ACCEPTABLE)]
    #[message("The User is already registered to the event")]
    UserAlreadyRegistered,

    #[status_code(FORBIDDEN)]
    #[message("User is not in Event")]
    UserNotInEvent,

    #[status_code(FORBIDDEN)]
    #[message("Change guest not possible")]
    ChangeGuestsDenied,
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

