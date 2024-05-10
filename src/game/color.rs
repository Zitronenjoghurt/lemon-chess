use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum Color {
    WHITE = 0,
    BLACK = 1,
    NONE = 2,
}

impl From<usize> for Color {
    fn from(number: usize) -> Self {
        match number {
            0 => Color::WHITE,
            1 => Color::BLACK,
            _ => Color::NONE,
        }
    }
}

impl Color {
    pub fn opponent_color(&self) -> Color {
        if self == &Color::WHITE {
            Color::BLACK
        } else {
            Color::WHITE
        }
    }

    pub fn get_fen_letter(&self) -> char {
        match self {
            Color::WHITE => 'w',
            Color::BLACK => 'b',
            Color::NONE => '-',
        }
    }

    pub fn from_fen_letter(letter: char) -> Self {
        match letter.to_ascii_lowercase() {
            'w' => Color::WHITE,
            'b' => Color::BLACK,
            _ => Color::NONE,
        }
    }
}
