use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::fmt;

use crate::game::error::GameError;

#[derive(Debug)]
pub enum ApiError {
    AuthorizationError(String),
    BadRequest(String),
    DatabaseError(String),
    NoPermission(String),
    NotFound(String),
    ParseError(String),
    RateLimited(u64),
    SerializationError(String),
    ServerError(String),
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

impl From<mongodb::bson::oid::Error> for ApiError {
    fn from(error: mongodb::bson::oid::Error) -> Self {
        ApiError::SerializationError(error.to_string())
    }
}

impl From<std::io::Error> for ApiError {
    fn from(error: std::io::Error) -> Self {
        ApiError::ServerError(error.to_string())
    }
}

impl From<gif::EncodingError> for ApiError {
    fn from(error: gif::EncodingError) -> Self {
        ApiError::ServerError(error.to_string())
    }
}

impl From<image::ImageError> for ApiError {
    fn from(error: image::ImageError) -> Self {
        ApiError::ServerError(error.to_string())
    }
}

impl From<GameError> for ApiError {
    fn from(error: GameError) -> Self {
        match error {
            GameError::DecodingError(message) => Self::ParseError(message),
            GameError::EncodingError(message) => Self::ParseError(message),
            GameError::ParseError(message) => Self::ParseError(message),
            GameError::ValidationError(message) => Self::BadRequest(message),
            GameError::AiError(message) => Self::ServerError(message),
        }
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
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            ApiError::NoPermission(message) => (StatusCode::FORBIDDEN, message),
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            ApiError::ParseError(message) => (StatusCode::BAD_REQUEST, message),
            ApiError::RateLimited(time_left_nanos) => (
                StatusCode::TOO_MANY_REQUESTS,
                format!("Cooldown: {}nanos", time_left_nanos),
            ),
            ApiError::ServerError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };

        (status, error_message).into_response()
    }
}
