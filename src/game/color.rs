#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    BLACK = 0,
    WHITE = 1,
    NONE = 2,
}

impl From<usize> for Color {
    fn from(number: usize) -> Self {
        match number {
            0 => Color::BLACK,
            1 => Color::WHITE,
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
}
