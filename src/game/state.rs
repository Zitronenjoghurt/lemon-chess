use crate::game::{bit_board::BitBoard, chess_board::ChessBoard};
use serde::Serialize;

use super::{color::Color, error::GameError, piece::Piece};

#[derive(Debug, Serialize)]
pub struct GameState {
    chess_board: ChessBoard,
    /// A mask including all occupied cells
    occupancy_mask: BitBoard,
    /// Initial pawn locations by color, 0 = black, 1 = white
    initial_pawn_masks: [BitBoard; 2],
}

impl Default for GameState {
    fn default() -> Self {
        let board = ChessBoard::default();
        Self {
            initial_pawn_masks: [
                board.mask_by_piece_and_color(Piece::PAWN, Color::BLACK),
                board.mask_by_piece_and_color(Piece::PAWN, Color::WHITE),
            ],
            chess_board: board,
            occupancy_mask: Default::default(),
        }
    }
}

impl GameState {
    pub fn make_move(&mut self, from: u8, to: u8) -> Result<bool, GameError> {
        let success = self.chess_board.make_move(from, to)?;
        if !success {
            return Ok(false);
        }

        self.update_occupancy_mask();

        Ok(true)
    }

    pub fn update_occupancy_mask(&mut self) {
        self.occupancy_mask = self.chess_board.colors[0] | self.chess_board.colors[1];
    }
}
