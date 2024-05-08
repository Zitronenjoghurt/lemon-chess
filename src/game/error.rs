use std::fmt;

#[derive(Debug)]
pub enum GameError {
    DecodingError(String),
    EncodingError(String),
    ValidationError(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<base64::DecodeError> for GameError {
    fn from(error: base64::DecodeError) -> Self {
        GameError::EncodingError(error.to_string())
    }
}
