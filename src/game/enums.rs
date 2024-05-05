#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum Piece {
    PAWN = 0,
    BISHOP = 1,
    KNIGHT = 2,
    ROOK = 3,
    QUEEN = 4,
    KING = 5,
    NONE = 6,
}

impl From<usize> for Piece {
    fn from(number: usize) -> Self {
        match number {
            0 => Piece::PAWN,
            1 => Piece::BISHOP,
            2 => Piece::KNIGHT,
            3 => Piece::ROOK,
            4 => Piece::QUEEN,
            5 => Piece::KING,
            _ => Piece::NONE,
        }
    }
}
