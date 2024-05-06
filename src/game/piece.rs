use super::bit_board::BitBoard;

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

impl Piece {
    pub fn get_move_mask(
        &self,
        index: u8,
        block_mask: BitBoard,
        initial_pawn_mask: BitBoard,
    ) -> BitBoard {
        let mut move_mask = BitBoard::default();
        match self {
            Piece::PAWN => {
                if initial_pawn_mask.get_bit(index) {
                    move_mask.populate_up(index, 2, block_mask);
                } else {
                    move_mask.populate_up(index, 1, block_mask)
                }
            }
            Piece::BISHOP => todo!(),
            Piece::KNIGHT => todo!(),
            Piece::ROOK => todo!(),
            Piece::QUEEN => todo!(),
            Piece::KING => todo!(),
            Piece::NONE => todo!(),
        }
        move_mask
    }
}
