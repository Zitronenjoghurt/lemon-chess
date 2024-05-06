use serde::Serialize;

use super::{bit_board::BitBoard, color::Color};

#[derive(Debug, PartialEq, Clone, Copy)]
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

    pub fn get_action_mask(
        &self,
        index: u8,
        opponent_color: Color,
        initial_pawn_mask: BitBoard,
        color_masks: [BitBoard; 2],
    ) -> BitBoard {
        let (move_mask, attack_mask) =
            self.get_move_and_attack_mask(index, opponent_color, initial_pawn_mask, color_masks);
        move_mask | attack_mask
    }

    /// Calculating move and attack mask together prevents mutliple calculation of the reach mask
    pub fn get_move_and_attack_mask(
        &self,
        index: u8,
        opponent_color: Color,
        initial_pawn_mask: BitBoard,
        color_masks: [BitBoard; 2],
    ) -> (BitBoard, BitBoard) {
        let block_mask = color_masks[0] | color_masks[1];
        let reach_mask = self.get_reach_mask(index, block_mask, initial_pawn_mask);
        let move_mask = Self::get_move_mask(reach_mask, color_masks);
        let attack_mask = Self::get_attack_mask(
            index,
            reach_mask,
            self,
            color_masks[opponent_color as usize],
        );
        (move_mask, attack_mask)
    }

    pub fn get_move_mask(reach_mask: BitBoard, color_masks: [BitBoard; 2]) -> BitBoard {
        // Subtract all player pieces from the reach mask
        reach_mask & !color_masks[0] & !color_masks[1]
    }

    pub fn get_attack_mask(
        index: u8,
        reach_mask: BitBoard,
        piece: &Piece,
        opponent_mask: BitBoard,
    ) -> BitBoard {
        if piece == &Piece::PAWN {
            let mut mask = BitBoard::default();
            mask.populate_up_left(index, 1, BitBoard::default());
            mask.populate_up_right(index, 1, BitBoard::default());
            mask & opponent_mask
        } else {
            reach_mask & opponent_mask
        }
    }
}
