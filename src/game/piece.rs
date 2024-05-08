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
        current_color: Color,
        block_mask: BitBoard,
        initial_pawn_mask: BitBoard,
    ) -> BitBoard {
        let mut mask = BitBoard::default();
        match self {
            Piece::PAWN => {
                let steps = if initial_pawn_mask.get_bit(index) {
                    2
                } else {
                    1
                };
                if current_color == Color::WHITE {
                    mask.populate_up(index, steps, block_mask);
                } else {
                    mask.populate_down(index, steps, block_mask)
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
        current_color: Color,
        initial_pawn_mask: BitBoard,
        color_masks: [BitBoard; 2],
        en_passant_indices: &[u8; 2],
    ) -> BitBoard {
        let (move_mask, attack_mask) = self.get_move_and_attack_mask(
            index,
            current_color,
            initial_pawn_mask,
            color_masks,
            en_passant_indices,
        );
        move_mask | attack_mask
    }

    /// Calculating move and attack mask together prevents mutliple calculation of the reach mask
    pub fn get_move_and_attack_mask(
        &self,
        index: u8,
        current_color: Color,
        initial_pawn_mask: BitBoard,
        color_masks: [BitBoard; 2],
        en_passant_indices: &[u8; 2],
    ) -> (BitBoard, BitBoard) {
        let block_mask = color_masks[0] | color_masks[1];
        let reach_mask = self.get_reach_mask(index, current_color, block_mask, initial_pawn_mask);
        let move_mask = Self::get_move_mask(reach_mask, color_masks);
        let attack_mask = Self::get_attack_mask(
            index,
            reach_mask,
            self,
            current_color,
            color_masks[current_color.opponent_color() as usize],
            en_passant_indices[current_color.opponent_color() as usize],
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
        current_color: Color,
        opponent_mask: BitBoard,
        en_passant_index: u8,
    ) -> BitBoard {
        if piece == &Piece::PAWN {
            let mut mask = BitBoard::default();
            if current_color == Color::WHITE {
                mask.populate_up_left(index, 1, BitBoard::default());
                mask.populate_up_right(index, 1, BitBoard::default());
            } else {
                mask.populate_down_left(index, 1, BitBoard::default());
                mask.populate_down_right(index, 1, BitBoard::default());
            }

            mask & (opponent_mask + en_passant_index)
        } else {
            reach_mask & opponent_mask
        }
    }

    // If a respective opponent piece is at a 1 position, it means the king is check
    pub fn get_king_threat_masks(
        index: u8,
        current_color: Color,
        block_mask: BitBoard,
    ) -> [BitBoard; 6] {
        let mut masks: [BitBoard; 6] = [BitBoard::default(); 6];

        for piece_id in 0..6 {
            let piece = Piece::from(piece_id);
            let reach_mask =
                piece.get_reach_mask(index, current_color, block_mask, BitBoard::default());
            let threat_mask = Piece::get_attack_mask(
                index,
                reach_mask,
                &piece,
                current_color,
                BitBoard(u64::MAX),
                64,
            );
            masks[piece_id] = threat_mask;
        }

        masks
    }

    pub fn get_name(&self) -> String {
        match self {
            Piece::PAWN => "Pawn".to_string(),
            Piece::BISHOP => "Bishop".to_string(),
            Piece::KNIGHT => "Knight".to_string(),
            Piece::ROOK => "Rook".to_string(),
            Piece::QUEEN => "Queen".to_string(),
            Piece::KING => "King".to_string(),
            Piece::NONE => "None".to_string(),
        }
    }

    pub fn get_image_name(&self, color: Color) -> String {
        let name = self.get_name();
        if color == Color::WHITE {
            format!("W_{}.png", name)
        } else {
            format!("B_{}.png", name)
        }
    }
}
