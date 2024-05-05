use std::fmt;

#[derive(Debug)]
pub enum GameError {
    ValidationError(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
