use crate::game::{bit_board::BitBoard, chess_board::ChessBoard};
use serde::Serialize;

use super::error::GameError;

#[derive(Debug, Default, Serialize)]
pub struct GameState {
    chess_board: ChessBoard,
    /// A mask including all occupied cells
    occupancy_mask: BitBoard,
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
