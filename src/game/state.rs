use crate::game::{bit_board::BitBoard, chess_board::ChessBoard};
use serde::{Deserialize, Serialize};

use super::{
    chess_board::AvailableMoves, color::Color, error::GameError, piece::Piece, position::Position,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    pub chess_board: ChessBoard,
    /// Next to move, 0 = white, 1 = black
    next_to_move: u8,
    half_move_counter: u8,
    full_move_counter: u8,
    /// Initial pawn locations by color, 0 = white, 1 = black
    initial_pawn_masks: [BitBoard; 2],
    /// All available moves (also by color)
    pub available_moves: [AvailableMoves; 2],
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
            board.mask_by_piece_and_color(Piece::PAWN, Color::WHITE),
            board.mask_by_piece_and_color(Piece::PAWN, Color::BLACK),
        ];

        let mut game_state = Self {
            initial_pawn_masks,
            chess_board: board,
            next_to_move: 0,
            half_move_counter: 0,
            full_move_counter: 1,
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

    pub fn to_fen(&self) -> String {
        let fen_positions = self.chess_board.to_fen_positions();

        let color_to_move = Color::from(self.next_to_move as usize);
        let active_color = color_to_move.get_fen_letter();

        let mut castling_rights = String::new();
        if self.kingside_castling_rights[Color::WHITE as usize] {
            castling_rights.push('K');
        }
        if self.queenside_castling_rights[Color::WHITE as usize] {
            castling_rights.push('Q');
        }
        if self.kingside_castling_rights[Color::BLACK as usize] {
            castling_rights.push('k');
        }
        if self.queenside_castling_rights[Color::BLACK as usize] {
            castling_rights.push('q');
        }

        if castling_rights.is_empty() {
            castling_rights = "-".to_string();
        }

        let en_passant_cell = self.en_passant_indices[color_to_move.opponent_color() as usize];
        let en_passant = if en_passant_cell == 64 {
            "-".to_string()
        } else {
            Position::try_from(en_passant_cell).unwrap().as_str()
        };

        format!(
            "{} {} {} {} {} {}",
            fen_positions,
            active_color,
            castling_rights,
            en_passant,
            self.half_move_counter,
            self.full_move_counter
        )
    }

    pub fn from_fen(fen: &str) -> Result<Self, GameError> {
        let parts: Vec<&str> = fen.split(' ').collect();
        if parts.len() != 6 {
            return Err(GameError::DecodingError(
                "FEN-String needs 6 components".to_string(),
            ));
        };

        let chess_board = ChessBoard::from_fen_positions(parts[0])?;

        let active_char = parts[1].chars().next().unwrap_or_default();
        let active_color = Color::from_fen_letter(active_char);

        let white_kingside_castling_right = parts[2].contains('K');
        let white_queenside_castling_right = parts[2].contains('Q');
        let black_kingside_castling_right = parts[2].contains('k');
        let black_queenside_castling_right = parts[2].contains('q');

        let kingside_castling_rights =
            [white_kingside_castling_right, black_kingside_castling_right];
        let queenside_castling_rights = [
            white_queenside_castling_right,
            black_queenside_castling_right,
        ];

        let white_en_passent = if active_color == Color::WHITE || parts[3] == "-" {
            64
        } else {
            Position::try_from(parts[3].to_string())? as u8
        };

        let black_en_passent = if active_color == Color::BLACK || parts[3] == "-" {
            64
        } else {
            Position::try_from(parts[3].to_string())? as u8
        };

        let half_move_counter = parts[4].parse::<u8>()?;
        let full_move_counter = parts[5].parse::<u8>()?;

        // Since its FEN, the pawns will always be in the same rows
        let initial_pawn_masks = [
            BitBoard(0b0000000000000000000000000000000000000000000000001111111100000000),
            BitBoard(0b0000000011111111000000000000000000000000000000000000000000000000),
        ];

        // Castling king/rook indices, important to comply with Fischer960 king and rook positions
        let white_king = if kingside_castling_rights[Color::WHITE as usize]
            || queenside_castling_rights[Color::WHITE as usize]
        {
            chess_board.get_king_position_by_color(Color::WHITE)
        } else {
            4
        };

        let black_king = if kingside_castling_rights[Color::BLACK as usize]
            || queenside_castling_rights[Color::BLACK as usize]
        {
            chess_board.get_king_position_by_color(Color::BLACK)
        } else {
            60
        };

        let white_kingside_rook = if kingside_castling_rights[Color::WHITE as usize] {
            let (_, rook) = chess_board
                .get_kingside_rook(Color::WHITE)
                .unwrap_or((4, 7));
            rook
        } else {
            7
        };

        let black_kingside_rook = if kingside_castling_rights[Color::BLACK as usize] {
            let (_, rook) = chess_board
                .get_kingside_rook(Color::BLACK)
                .unwrap_or((60, 63));
            rook
        } else {
            63
        };

        let white_queenside_rook = if queenside_castling_rights[Color::WHITE as usize] {
            let (_, rook) = chess_board
                .get_queenside_rook(Color::WHITE)
                .unwrap_or((4, 0));
            rook
        } else {
            0
        };

        let black_queenside_rook = if queenside_castling_rights[Color::BLACK as usize] {
            let (_, rook) = chess_board
                .get_queenside_rook(Color::BLACK)
                .unwrap_or((60, 56));
            rook
        } else {
            56
        };

        let mut state = Self {
            initial_pawn_masks,
            chess_board,
            next_to_move: active_color as u8,
            half_move_counter,
            full_move_counter,
            available_moves: Default::default(),
            check_states: [false, false],
            en_passant_indices: [white_en_passent, black_en_passent],
            kingside_castling_rights,
            queenside_castling_rights,
            can_castle_kingside: [false, false],
            can_castle_queenside: [false, false],
            king_indices: [white_king, black_king],
            kingside_rook_indices: [white_kingside_rook, black_kingside_rook],
            queenside_rook_indices: [white_queenside_rook, black_queenside_rook],
        };

        state.update()?;

        Ok(state)
    }

    pub fn make_move(&mut self, from: u8, to: u8) -> Result<bool, GameError> {
        let (success, capture_or_pawn_move) = self.chess_board.make_move(
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
        self.clock(capture_or_pawn_move);

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
        self.clock(false);

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
        self.clock(false);

        Ok(true)
    }

    /// Handles ticking move counter and switching active player
    pub fn clock(&mut self, capture_or_pawn_move: bool) {
        if Color::from(self.next_to_move as usize) == Color::BLACK {
            self.full_move_counter += 1;
        }

        if capture_or_pawn_move {
            self.half_move_counter = 0;
        } else {
            self.half_move_counter += 1;
        }

        if self.next_to_move == 1 {
            self.next_to_move = 0;
        } else {
            self.next_to_move = 1;
        }
    }

    pub fn update(&mut self) -> Result<(), GameError> {
        self.update_check_states();
        self.update_legal_moves()?;
        self.update_castle_ability();
        Ok(())
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
