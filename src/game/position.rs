use serde::{Deserialize, Serialize};

use super::error::GameError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
// Prioritizing speed, its faster to just map all 64 coordinates to the respective index
pub enum Position {
    A1,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
    A2,
    B2,
    C2,
    D2,
    E2,
    F2,
    G2,
    H2,
    A3,
    B3,
    C3,
    D3,
    E3,
    F3,
    G3,
    H3,
    A4,
    B4,
    C4,
    D4,
    E4,
    F4,
    G4,
    H4,
    A5,
    B5,
    C5,
    D5,
    E5,
    F5,
    G5,
    H5,
    A6,
    B6,
    C6,
    D6,
    E6,
    F6,
    G6,
    H6,
    A7,
    B7,
    C7,
    D7,
    E7,
    F7,
    G7,
    H7,
    A8,
    B8,
    C8,
    D8,
    E8,
    F8,
    G8,
    H8,
}

impl Position {
    pub fn as_str(&self) -> String {
        let index = *self as u8;
        let file = b'A' + (index % 8);
        let rank = 1 + (index / 8);
        format!("{}{}", file as char, rank)
    }
}

impl std::convert::TryFrom<String> for Position {
    type Error = GameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            return Err(GameError::DecodingError(format!(
                "Invalid position '{}'. Position must be 2 characters long.",
                value
            )));
        }

        let mut chars = value.chars();
        let file_char = chars.next().unwrap_or_default();
        let rank_char = chars.next().unwrap_or_default();

        if !file_char.is_ascii_alphabetic() || !rank_char.is_ascii_digit() {
            return Err(GameError::DecodingError(format!(
                "Invalid position format: '{}'.",
                value
            )));
        }

        let file = file_char.to_ascii_uppercase() as u8 - b'A';
        let rank = rank_char.to_digit(10).unwrap_or_default() as u8 - 1;

        if file > 7 || rank > 7 {
            return Err(GameError::DecodingError(format!(
                "Position out of bounds: '{}'.",
                value
            )));
        }

        let index = rank * 8 + file;
        Self::try_from(index)
            .map_err(|_| GameError::DecodingError(format!("Invalid position index: '{}'.", index)))
    }
}

impl From<Position> for u8 {
    #[inline]
    fn from(val: Position) -> Self {
        val as u8
    }
}

impl std::convert::TryFrom<u8> for Position {
    type Error = GameError;

    fn try_from(index: u8) -> Result<Self, Self::Error> {
        if index < 64 {
            Ok(unsafe { std::mem::transmute::<u8, Position>(index) })
        } else {
            Err(GameError::DecodingError(format!(
                "Can't create position from index {}. Index has to be between 0 and 63.",
                index
            )))
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Move(pub Position, pub Position);

impl From<Move> for String {
    fn from(m: Move) -> Self {
        format!("{}->{}", m.0.as_str(), m.1.as_str())
    }
}
