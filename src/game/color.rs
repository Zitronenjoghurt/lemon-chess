#[derive(Debug, PartialEq, Clone, Copy)]
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
}
