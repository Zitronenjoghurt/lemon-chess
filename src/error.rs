use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    AuthorizationError(String),
    DatabaseError(String),
    SerializationError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<mongodb::error::Error> for ApiError {
    fn from(error: mongodb::error::Error) -> Self {
        ApiError::DatabaseError(error.to_string())
    }
}

impl From<mongodb::bson::ser::Error> for ApiError {
    fn from(error: mongodb::bson::ser::Error) -> Self {
        ApiError::SerializationError(error.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::DatabaseError(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("A database error occurred: {}", message),
            ),
            ApiError::SerializationError(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("A serialization error occured: {}", message),
            ),
            ApiError::AuthorizationError(message) => (
                StatusCode::UNAUTHORIZED,
                format!("An authorization error occured: {}", message),
            ),
        };

        (status, error_message).into_response()
    }
}
