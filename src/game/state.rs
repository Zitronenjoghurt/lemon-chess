use crate::game::{bit_board::BitBoard, chess_board::ChessBoard};
use serde::Serialize;

use super::{chess_board::AvailableMoves, color::Color, error::GameError, piece::Piece};

#[derive(Debug, Serialize)]
pub struct GameState {
    chess_board: ChessBoard,
    /// A mask including all occupied cells
    occupancy_mask: BitBoard,
    /// Initial pawn locations by color, 0 = black, 1 = white
    initial_pawn_masks: [BitBoard; 2],
    /// All available moves (also by color)
    available_moves: [AvailableMoves; 2],
}

impl GameState {
    pub fn new() -> Result<Self, GameError> {
        let board = ChessBoard::default();
        let initial_pawn_masks = [
            board.mask_by_piece_and_color(Piece::PAWN, Color::BLACK),
            board.mask_by_piece_and_color(Piece::PAWN, Color::WHITE),
        ];

        let mut game_state = Self {
            initial_pawn_masks,
            chess_board: board,
            occupancy_mask: Default::default(),
            available_moves: Default::default(),
        };

        game_state.update()?;

        Ok(game_state)
    }

    pub fn make_move(&mut self, from: u8, to: u8) -> Result<bool, GameError> {
        let success = self.chess_board.make_move(from, to)?;
        if !success {
            return Ok(false);
        }

        self.update()?;

        Ok(true)
    }

    pub fn update(&mut self) -> Result<(), GameError> {
        self.update_occupancy_mask();
        self.update_legal_moves()?;
        Ok(())
    }

    pub fn update_occupancy_mask(&mut self) {
        self.occupancy_mask = self.chess_board.colors[0] | self.chess_board.colors[1];
    }

    pub fn get_legal_moves(&self, color: Color) -> Result<AvailableMoves, GameError> {
        self.chess_board
            .generate_legal_moves(color, self.initial_pawn_masks[color as usize])
    }

    pub fn update_legal_moves(&mut self) -> Result<(), GameError> {
        self.available_moves[0] = self.get_legal_moves(Color::BLACK)?;
        self.available_moves[1] = self.get_legal_moves(Color::WHITE)?;
        Ok(())
    }
}
