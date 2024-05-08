use crate::game::{bit_board::BitBoard, chess_board::ChessBoard};
use serde::Serialize;

use super::{
    chess_board::AvailableMoves,
    color::{self, Color},
    error::GameError,
    piece::Piece,
};

#[derive(Debug, Serialize)]
pub struct GameState {
    pub chess_board: ChessBoard,
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
    /// Castle rights by color
    kingside_castling_rights: [bool; 2],
    queenside_castling_rights: [bool; 2],
    /// Castle abilities by color
    can_castle_kingside: [bool; 2],
    can_castle_queenside: [bool; 2],
    /// Rook & king positions by color
    king_indices: [u8; 2],
    kingside_rook_indices: [u8; 2],
    queenside_rook_indices: [u8; 2],
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
            kingside_castling_rights: [true, true],
            queenside_castling_rights: [true, true],
            can_castle_kingside: [false, false],
            can_castle_queenside: [false, false],
            // Uses default board, therefore positions are predictable
            king_indices: [4, 60],
            kingside_rook_indices: [7, 63],
            queenside_rook_indices: [0, 56],
        };

        game_state.update()?;

        Ok(game_state)
    }

    pub fn make_move(&mut self, from: u8, to: u8) -> Result<bool, GameError> {
        let success = self.chess_board.make_move(
            from,
            to,
            &mut self.en_passant_indices,
            &mut self.kingside_castling_rights,
            &mut self.queenside_castling_rights,
        )?;
        if !success {
            return Ok(false);
        }

        self.update()?;

        Ok(true)
    }

    pub fn castle_kingside(&mut self, color: Color) -> Result<bool, GameError> {
        if !self.can_castle_kingside[color as usize] {
            return Ok(false);
        }

        self.chess_board.castle_kingside(
            self.king_indices[color as usize],
            self.kingside_rook_indices[color as usize],
        )?;

        self.kingside_castling_rights[color as usize] = false;
        self.queenside_castling_rights[color as usize] = false;

        self.update()?;
        Ok(true)
    }

    pub fn castle_queenside(&mut self, color: Color) -> Result<bool, GameError> {
        if !self.can_castle_queenside[color as usize] {
            return Ok(false);
        }

        self.chess_board.castle_queenside(
            self.king_indices[color as usize],
            self.queenside_rook_indices[color as usize],
        )?;

        self.kingside_castling_rights[color as usize] = false;
        self.queenside_castling_rights[color as usize] = false;

        self.update()?;
        Ok(true)
    }

    pub fn update(&mut self) -> Result<(), GameError> {
        self.update_occupancy_mask();
        self.update_check_states();
        self.update_legal_moves()?;
        self.update_castle_ability();
        Ok(())
    }

    pub fn update_occupancy_mask(&mut self) {
        self.occupancy_mask = self.chess_board.colors[0] | self.chess_board.colors[1];
    }

    pub fn update_check_states(&mut self) {
        self.check_states[Color::BLACK as usize] = self.chess_board.is_king_check(Color::BLACK);
        self.check_states[Color::WHITE as usize] = self.chess_board.is_king_check(Color::WHITE);
    }

    pub fn update_castle_ability(&mut self) {
        for color_index in 0..2 {
            let color = Color::from(color_index);

            self.can_castle_kingside[color_index] = self.kingside_castling_rights[color_index]
                && self.chess_board.can_castle_kingside(color);

            self.can_castle_queenside[color_index] = self.queenside_castling_rights[color_index]
                && self.chess_board.can_castle_queenside(color);
        }
    }

    pub fn get_legal_moves(&mut self, color: Color) -> Result<AvailableMoves, GameError> {
        self.chess_board.generate_legal_moves(
            color,
            self.initial_pawn_masks[color as usize],
            &self.en_passant_indices,
            &self.kingside_castling_rights,
            &self.queenside_castling_rights,
        )
    }

    pub fn update_legal_moves(&mut self) -> Result<(), GameError> {
        self.available_moves[0] = self.get_legal_moves(Color::WHITE)?;
        self.available_moves[1] = self.get_legal_moves(Color::BLACK)?;
        Ok(())
    }
}
