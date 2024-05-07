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
    /// Check state by color
    check_states: [bool; 2],
    /// The fields of en passant by color, 64 being the NONE state
    en_passant_indices: [u8; 2],
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
            check_states: [false, false],
            en_passant_indices: [64, 64],
        };

        game_state.update()?;

        Ok(game_state)
    }

    pub fn make_move(&mut self, from: u8, to: u8) -> Result<bool, GameError> {
        let success = self
            .chess_board
            .make_move(from, to, &mut self.en_passant_indices)?;
        if !success {
            return Ok(false);
        }

        self.update()?;

        Ok(true)
    }

    pub fn update(&mut self) -> Result<(), GameError> {
        self.update_occupancy_mask();
        self.update_check_states();
        self.update_legal_moves()?;
        Ok(())
    }

    pub fn update_occupancy_mask(&mut self) {
        self.occupancy_mask = self.chess_board.colors[0] | self.chess_board.colors[1];
    }

    pub fn update_check_states(&mut self) {
        self.check_states[Color::BLACK as usize] = self.chess_board.is_king_check(Color::BLACK);
        self.check_states[Color::WHITE as usize] = self.chess_board.is_king_check(Color::WHITE);
    }

    pub fn get_legal_moves(&mut self, color: Color) -> Result<AvailableMoves, GameError> {
        self.chess_board.generate_legal_moves(
            color,
            self.initial_pawn_masks[color as usize],
            &self.en_passant_indices,
        )
    }

    pub fn update_legal_moves(&mut self) -> Result<(), GameError> {
        self.available_moves[0] = self.get_legal_moves(Color::WHITE)?;
        self.available_moves[1] = self.get_legal_moves(Color::BLACK)?;
        Ok(())
    }
}
