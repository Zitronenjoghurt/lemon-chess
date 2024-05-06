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
    /// The reach mask will include the cell that the piece is blocked by
    /// That way you can just subtract the current players color mask from the reach mask to get the move mask
    /// Or AND the opponent color mask with the reach mask to get the attack mask (except pawns)
    pub fn get_reach_mask(
        &self,
        index: u8,
        block_mask: BitBoard,
        initial_pawn_mask: BitBoard,
    ) -> BitBoard {
        let mut mask = BitBoard::default();
        match self {
            Piece::PAWN => {
                if initial_pawn_mask.get_bit(index) {
                    mask.populate_up(index, 2, block_mask);
                } else {
                    mask.populate_up(index, 1, block_mask);
                }
            }
            Piece::BISHOP => mask.populate_diag(index, 7, block_mask),
            Piece::KNIGHT => {
                mask.populate_jump(index, 2, 1);
                mask.populate_jump(index, 1, 2);
                mask.populate_jump(index, 2, -1);
                mask.populate_jump(index, 1, -2);
                mask.populate_jump(index, -2, 1);
                mask.populate_jump(index, -1, 2);
                mask.populate_jump(index, -2, -1);
                mask.populate_jump(index, -1, -2);
            }
            Piece::ROOK => mask.populate_vert_hor(index, 7, block_mask),
            Piece::QUEEN => {
                mask.populate_vert_hor(index, 7, block_mask);
                mask.populate_diag(index, 7, block_mask);
            }
            Piece::KING => {
                mask.populate_vert_hor(index, 1, block_mask);
                mask.populate_diag(index, 1, block_mask);
            }
            Piece::NONE => {}
        }
        mask
    }
}
