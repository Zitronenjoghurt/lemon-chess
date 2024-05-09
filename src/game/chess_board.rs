use crate::game::{bit_board::BitBoard, color::Color, error::GameError, piece::Piece};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize)]
/// INFERENCES:
/// - Board is always 8x8
/// - Only 1 King
/// - Rooks always on either side of the king
/// - White is always at the bottom of the board (at least internally, board can be rotated for visualization)
pub struct ChessBoard {
    pub colors: [BitBoard; 2],
    pub pieces: [BitBoard; 6],
}

// Representation can later be individualized by rotating the board
// Index starts in the bottom left
impl Default for ChessBoard {
    fn default() -> Self {
        Self {
            colors: [
                BitBoard(0b0000000000000000000000000000000000000000000000001111111111111111),
                BitBoard(0b1111111111111111000000000000000000000000000000000000000000000000),
            ],
            pieces: [
                BitBoard(0b0000000011111111000000000000000000000000000000001111111100000000), // Pawn
                BitBoard(0b0010010000000000000000000000000000000000000000000000000000100100), // Bishop
                BitBoard(0b0100001000000000000000000000000000000000000000000000000001000010), // Knight
                BitBoard(0b1000000100000000000000000000000000000000000000000000000010000001), // Rook
                BitBoard(0b0000100000000000000000000000000000000000000000000000000000001000), // Queen
                BitBoard(0b0001000000000000000000000000000000000000000000000000000000010000), // King
            ],
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Clone, Debug, Default, Hash, Serialize)]
/// Describes all available moves with a location index and a vector of target indices
pub struct AvailableMoves(pub Vec<(u8, Vec<u8>)>);

impl ChessBoard {
    pub fn new_empty() -> Self {
        Self {
            colors: [BitBoard(0), BitBoard(0)],
            pieces: [
                BitBoard(0),
                BitBoard(0),
                BitBoard(0),
                BitBoard(0),
                BitBoard(0),
                BitBoard(0),
            ],
        }
    }

    pub fn validate_index(index: u8) -> Result<(), GameError> {
        if index >= 64 {
            return Err(GameError::ValidationError(
                "Board address out of range.".to_string(),
            ));
        }
        Ok(())
    }

    pub fn to_fen_positions(&self) -> String {
        let mut result = String::new();
        let mut row_string = String::new();
        let mut empty_cells: u8 = 0;
        for i in (0..64).rev() {
            let is_end_of_row = i % 8 == 0;
            let (piece, color) = self.piece_and_color_at_cell(i).unwrap();
            let has_piece = piece != Piece::NONE && color != Color::NONE;

            if !has_piece {
                empty_cells += 1;
                if !is_end_of_row {
                    continue;
                }
            }

            if empty_cells > 0 {
                row_string.push_str(&empty_cells.to_string());
                empty_cells = 0;
            }

            if has_piece {
                row_string.push_str(&piece.get_fen_letter(color));
            }

            if is_end_of_row {
                let reversed_row_string = row_string.chars().rev().collect::<String>();
                result.push_str(&reversed_row_string);
                row_string = String::new();

                // Prevents trailing / at the end
                if i != 0 {
                    result.push('/');
                }
            }
        }

        result
    }

    pub fn from_fen_positions(fen: &str) -> Result<Self, GameError> {
        let mut board = Self::new_empty();

        for (row_index, fen_row) in fen.split('/').enumerate() {
            if fen_row.is_empty() {
                continue;
            }

            if row_index >= 8 {
                return Err(GameError::DecodingError(
                    "FEN-String included too many rows".to_string(),
                ));
            }

            let row = 7 - row_index;
            let mut column: u32 = 0;
            for fen_char in fen_row.chars() {
                if let Some(digit) = fen_char.to_digit(10) {
                    column += digit;
                    continue;
                }

                if column >= 8 {
                    return Err(GameError::DecodingError(format!(
                        "Invalid digit in FEN-String in row {}",
                        (7 - row) + 1
                    )));
                }

                let (piece, color) = Piece::from_fen_letter(fen_char);
                if piece == Piece::NONE {
                    return Err(GameError::DecodingError(format!(
                        "Found invalid character in FEN-String: {}",
                        fen_char
                    )));
                }
                let index = ((row * 8) + column as usize) as u8;
                board.place_piece(index, piece, color)?;
                column += 1;
            }
        }

        Ok(board)
    }

    pub fn to_base64(&self) -> Result<String, GameError> {
        let mut bit_vec = Vec::new();

        for color_board in &self.colors {
            bit_vec.append(&mut color_board.0.to_be_bytes().to_vec());
        }
        for piece_board in &self.pieces {
            bit_vec.append(&mut piece_board.0.to_be_bytes().to_vec());
        }

        Ok(STANDARD.encode(&bit_vec))
    }

    pub fn from_base64(encoded: &str) -> Result<Self, GameError> {
        let decoded = STANDARD.decode(encoded)?;

        if decoded.len() != 64 {
            return Err(GameError::EncodingError("Invalid length.".to_string()));
        }

        let mut colors = [BitBoard(0); 2];
        let mut pieces = [BitBoard(0); 6];

        for (i, color_board) in colors.iter_mut().enumerate() {
            let start = i * 8;
            let bytes = &decoded[start..start + 8];
            color_board.0 = u64::from_be_bytes(bytes.try_into().unwrap());
        }

        for (i, piece_board) in pieces.iter_mut().enumerate() {
            let start = (2 + i) * 8;
            let bytes = &decoded[start..start + 8];
            piece_board.0 = u64::from_be_bytes(bytes.try_into().unwrap());
        }

        Ok(ChessBoard { colors, pieces })
    }

    pub fn is_cell_occupied(&self, index: u8) -> Result<bool, GameError> {
        Self::validate_index(index)?;
        let occupied_by_white = self.colors[0].get_bit(index);
        let occupied_by_black = self.colors[1].get_bit(index);
        Ok(occupied_by_black || occupied_by_white)
    }

    pub fn piece_at_cell(&self, index: u8) -> Result<Piece, GameError> {
        Self::validate_index(index)?;
        for piece_id in 0..6 {
            if self.pieces[piece_id].get_bit(index) {
                return Ok(Piece::from(piece_id));
            }
        }

        Ok(Piece::NONE)
    }

    pub fn color_at_cell(&self, index: u8) -> Result<Color, GameError> {
        Self::validate_index(index)?;
        for color_id in 0..2 {
            if self.colors[color_id].get_bit(index) {
                return Ok(Color::from(color_id));
            }
        }

        Ok(Color::NONE)
    }

    pub fn piece_and_color_at_cell(&self, index: u8) -> Result<(Piece, Color), GameError> {
        let piece = Self::piece_at_cell(self, index)?;
        let color = Self::color_at_cell(self, index)?;
        Ok((piece, color))
    }

    pub fn place_piece(&mut self, index: u8, piece: Piece, color: Color) -> Result<(), GameError> {
        Self::validate_index(index)?;
        if piece == Piece::NONE || color == Color::NONE {
            return Err(GameError::ValidationError(
                "Unable to place undefined piece or piece with undefined color".to_string(),
            ));
        }

        self.pieces[piece as usize].set_bit(index);
        self.colors[color as usize].set_bit(index);

        Ok(())
    }

    pub fn mask_by_piece_and_color(&self, piece: Piece, color: Color) -> BitBoard {
        self.pieces[piece as usize] & self.colors[color as usize]
    }

    pub fn relocate_piece(&mut self, from: u8, to: u8) -> Result<(), GameError> {
        Self::validate_index(to)?;
        let (piece, color) = self.piece_and_color_at_cell(from)?;
        self.pieces[piece as usize].clear_bit(from);
        self.pieces[piece as usize].set_bit(to);
        self.colors[color as usize].clear_bit(from);
        self.colors[color as usize].set_bit(to);
        Ok(())
    }

    pub fn make_move(
        &mut self,
        from: u8,
        to: u8,
        en_passant_indices: &mut [u8; 2],
        kingside_castling_rights: &mut [bool; 2],
        queenside_castling_rights: &mut [bool; 2],
    ) -> Result<bool, GameError> {
        Self::validate_index(from)?;
        Self::validate_index(to)?;

        let (source_piece, source_color) = Self::piece_and_color_at_cell(self, from)?;
        let (target_piece, target_color) = Self::piece_and_color_at_cell(self, to)?;
        if source_piece == Piece::NONE || target_color == source_color {
            return Ok(false);
        }

        // Capture piece
        if target_piece != Piece::NONE {
            self.pieces[target_piece as usize].clear_bit(to);
            self.colors[target_color as usize].clear_bit(to);
        }

        // Update piece
        let piece_index = source_piece as usize;
        self.pieces[piece_index].clear_bit(from);
        self.pieces[piece_index].set_bit(to);

        // Update color
        let color_index = source_color as usize;
        self.colors[color_index].clear_bit(from);
        self.colors[color_index].set_bit(to);

        // Capture en-passant
        let opponent_color = source_color.opponent_color();
        if source_piece == Piece::PAWN && to == en_passant_indices[opponent_color as usize] {
            let captured_pawn_index = match opponent_color {
                Color::BLACK => to - 8,
                Color::WHITE => to + 8,
                Color::NONE => to + 8,
            };
            self.pieces[Piece::PAWN as usize].clear_bit(captured_pawn_index);
            self.colors[opponent_color as usize].clear_bit(captured_pawn_index);
            en_passant_indices[opponent_color as usize] = 64;
        }

        // Update en-passant
        if source_piece == Piece::PAWN && to.abs_diff(from) == 16 {
            en_passant_indices[source_color as usize] = match source_color {
                Color::BLACK => to + 8,
                Color::WHITE => to - 8,
                Color::NONE => to - 8,
            }
        } else {
            en_passant_indices[source_color as usize] = 64;
        }

        // Update kingside castling rights
        if kingside_castling_rights[color_index] {
            // King has moved, castling rights removed
            if source_piece == Piece::KING {
                kingside_castling_rights[color_index] = false;
                queenside_castling_rights[color_index] = false;
            } else if source_piece == Piece::ROOK {
                let king_index = self.get_king_position_by_color(source_color);
                // rook is kingside
                if from > king_index {
                    kingside_castling_rights[color_index] = false;
                }
            }
        }

        // Update queenside castling rights
        if queenside_castling_rights[color_index] && source_piece == Piece::ROOK {
            let king_index = self.get_king_position_by_color(source_color);
            if from < king_index {
                queenside_castling_rights[color_index] = false;
            }
        }

        Ok(true)
    }

    pub fn castle_kingside(&mut self, king_index: u8, rook_index: u8) -> Result<(), GameError> {
        Self::validate_index(king_index)?;
        Self::validate_index(rook_index)?;

        let color = self.color_at_cell(king_index)?;
        let new_king_index = match color {
            Color::WHITE => 6,
            Color::BLACK => 62,
            Color::NONE => 1,
        };
        let new_rook_index = new_king_index - 1;

        self.relocate_piece(king_index, new_king_index)?;
        self.relocate_piece(rook_index, new_rook_index)?;

        Ok(())
    }

    pub fn castle_queenside(&mut self, king_index: u8, rook_index: u8) -> Result<(), GameError> {
        Self::validate_index(king_index)?;
        Self::validate_index(rook_index)?;

        let color = self.color_at_cell(king_index)?;
        let new_king_index = match color {
            Color::WHITE => 2,
            Color::BLACK => 58,
            Color::NONE => 1,
        };
        let new_rook_index = new_king_index + 1;

        self.relocate_piece(king_index, new_king_index)?;
        self.relocate_piece(rook_index, new_rook_index)?;

        Ok(())
    }

    pub fn generate_legal_moves(
        &self,
        color: Color,
        initial_pawn_mask: BitBoard,
        en_passant_indices: &[u8; 2],
        kingside_castling_rights: &[bool; 2],
        queenside_castling_rights: &[bool; 2],
    ) -> Result<AvailableMoves, GameError> {
        let piece_indices = self.colors[color as usize].get_bits();
        let mut piece_moves: Vec<(u8, Vec<u8>)> = Vec::new();

        for index in piece_indices {
            let piece = self.piece_at_cell(index)?;
            let action_mask = piece.get_action_mask(
                index,
                color,
                initial_pawn_mask,
                self.colors,
                en_passant_indices,
            );

            let target_indices = action_mask.get_bits();
            let mut valid_targets: Vec<u8> = Vec::new();
            for target_index in target_indices {
                if !Self::does_move_lead_to_check(
                    self,
                    color,
                    index,
                    target_index,
                    en_passant_indices,
                    kingside_castling_rights,
                    queenside_castling_rights,
                ) {
                    valid_targets.push(target_index)
                }
            }
            piece_moves.push((index, valid_targets))
        }

        Ok(AvailableMoves(piece_moves))
    }

    /// If a move leads to your own king being in check
    pub fn does_move_lead_to_check(
        &self,
        color: Color,
        from: u8,
        to: u8,
        en_passant_indices: &[u8; 2],
        kingside_castling_rights: &[bool; 2],
        queenside_castling_rights: &[bool; 2],
    ) -> bool {
        let mut future_board = self.clone();
        let mut future_en_passant_indices = *en_passant_indices;
        let mut future_kingside_castling_rights = *kingside_castling_rights;
        let mut future_queenside_castling_rights = *queenside_castling_rights;
        if let Ok(success) = future_board.make_move(
            from,
            to,
            &mut future_en_passant_indices,
            &mut future_kingside_castling_rights,
            &mut future_queenside_castling_rights,
        ) {
            if success {
                return future_board.is_king_check(color);
            }
        }
        false
    }

    pub fn get_king_position_by_color(&self, color: Color) -> u8 {
        let king_board = Self::mask_by_piece_and_color(self, Piece::KING, color);
        let king_positions = king_board.get_bits();
        if king_positions.is_empty() {
            return 0; // undefined behavior
        }
        king_positions[0]
    }

    pub fn get_king_check_positions(&self, color: Color) -> Vec<u8> {
        let king_index = self.get_king_position_by_color(color);
        let block_mask = self.colors[0] | self.colors[1];
        let threat_masks = Piece::get_king_threat_masks(king_index, color, block_mask);

        let mut check_positions: Vec<u8> = Vec::new();
        for i in 0..6 {
            let threats =
                threat_masks[i] & self.pieces[i] & self.colors[color.opponent_color() as usize];
            check_positions.extend(threats.get_bits());
        }
        check_positions
    }

    pub fn is_king_check(&self, color: Color) -> bool {
        !self.get_king_check_positions(color).is_empty()
    }

    pub fn get_attack_mask_by_color(&self, color: Color) -> BitBoard {
        let block_mask = self.colors[0] | self.colors[1];
        let mut final_mask = BitBoard::default();
        for piece_id in 0..6 {
            let piece = Piece::from(piece_id);
            let piece_indices = (self.pieces[piece_id] & self.colors[color as usize]).get_bits();
            for piece_index in piece_indices {
                let reach_mask =
                    piece.get_reach_mask(piece_index, color, block_mask, BitBoard::default());
                let full_attack_mask = Piece::get_attack_mask(
                    piece_index,
                    reach_mask,
                    &piece,
                    color,
                    BitBoard(u64::MAX),
                    64,
                );
                final_mask = final_mask | full_attack_mask;
            }
        }
        final_mask
    }

    /// Returns king and kingside rook index
    pub fn get_kingside_rook(&self, color: Color) -> Option<(u8, u8)> {
        let king_index = self.get_king_position_by_color(color);
        let rook_board = self.pieces[Piece::ROOK as usize] & self.colors[color as usize];
        let rook_indices = rook_board.get_bits();
        match rook_indices.iter().max().copied() {
            Some(rook_index) => {
                if king_index > rook_index {
                    None
                } else {
                    Some((king_index, rook_index))
                }
            }
            None => None,
        }
    }

    /// Returns queen and queenside rook index
    pub fn get_queenside_rook(&self, color: Color) -> Option<(u8, u8)> {
        let king_index = self.get_king_position_by_color(color);
        let rook_board = self.pieces[Piece::ROOK as usize] & self.colors[color as usize];
        let rook_indices = rook_board.get_bits();
        match rook_indices.iter().min().copied() {
            Some(rook_index) => {
                if king_index < rook_index {
                    None
                } else {
                    Some((king_index, rook_index))
                }
            }
            None => None,
        }
    }

    pub fn can_castle_kingside(&self, color: Color) -> bool {
        let (king_index, rook_index) = match self.get_kingside_rook(color) {
            Some(indices) => indices,
            None => return false,
        };
        self.can_castle_common(color, &king_index, rook_index)
    }

    pub fn can_castle_queenside(&self, color: Color) -> bool {
        let (king_index, rook_index) = match self.get_queenside_rook(color) {
            Some(indices) => indices,
            None => return false,
        };
        self.can_castle_common(color, &king_index, rook_index)
    }

    pub fn can_castle_common(&self, color: Color, king_index: &u8, rook_index: u8) -> bool {
        let block_mask = self.colors[0] | self.colors[1];
        let rook_reach_mask =
            Piece::ROOK.get_reach_mask(rook_index, color, block_mask, BitBoard::default());
        let rook_influence_cells = rook_reach_mask.get_bits();

        // Rook cant reach king
        if !rook_influence_cells.contains(king_index) {
            return false;
        }

        let opponent_attack_mask = self.get_attack_mask_by_color(color.opponent_color());
        let opponent_attack_indices = opponent_attack_mask.get_bits();
        for cell in rook_influence_cells {
            if opponent_attack_indices.contains(&cell) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use crate::game::position::Position as Pos;

    use super::*;

    #[test]
    fn test_base64() {
        let board = ChessBoard::default();
        let base64_string = board.to_base64().unwrap();
        let decoded_board = ChessBoard::from_base64(&base64_string).unwrap();
        assert_eq!(board, decoded_board);
    }

    #[test]
    fn test_is_cell_occupied() {
        let board = ChessBoard::default();
        assert!(board.is_cell_occupied(Pos::A1.into()).unwrap());
        assert!(board.is_cell_occupied(Pos::H2.into()).unwrap());
        assert!(board.is_cell_occupied(Pos::A7.into()).unwrap());
        assert!(board.is_cell_occupied(Pos::H8.into()).unwrap());
        assert!(!board.is_cell_occupied(Pos::A3.into()).unwrap());
        assert!(!board.is_cell_occupied(Pos::H6.into()).unwrap());
        assert!(board.is_cell_occupied(64).is_err());
    }

    #[test]
    fn test_piece_at_cell() {
        let board = ChessBoard::default();
        assert_eq!(board.piece_at_cell(Pos::A1.into()).unwrap(), Piece::ROOK);
        assert_eq!(board.piece_at_cell(Pos::B1.into()).unwrap(), Piece::KNIGHT);
        assert_eq!(board.piece_at_cell(Pos::C1.into()).unwrap(), Piece::BISHOP);
        assert_eq!(board.piece_at_cell(Pos::D1.into()).unwrap(), Piece::QUEEN);
        assert_eq!(board.piece_at_cell(Pos::E1.into()).unwrap(), Piece::KING);
        assert_eq!(board.piece_at_cell(Pos::F1.into()).unwrap(), Piece::BISHOP);
        assert_eq!(board.piece_at_cell(Pos::G1.into()).unwrap(), Piece::KNIGHT);
        assert_eq!(board.piece_at_cell(Pos::H1.into()).unwrap(), Piece::ROOK);
        assert_eq!(board.piece_at_cell(Pos::A2.into()).unwrap(), Piece::PAWN);
        assert_eq!(board.piece_at_cell(Pos::B2.into()).unwrap(), Piece::PAWN);
        assert_eq!(board.piece_at_cell(Pos::A3.into()).unwrap(), Piece::NONE);
        assert!(board.piece_at_cell(64).is_err());
    }

    #[test]
    fn test_color_at_cell() {
        let board = ChessBoard::default();
        assert_eq!(board.color_at_cell(Pos::A1.into()).unwrap(), Color::BLACK);
        assert_eq!(board.color_at_cell(Pos::H2.into()).unwrap(), Color::BLACK);
        assert_eq!(board.color_at_cell(Pos::A3.into()).unwrap(), Color::NONE);
        assert_eq!(board.color_at_cell(Pos::A7.into()).unwrap(), Color::WHITE);
        assert_eq!(board.color_at_cell(Pos::H8.into()).unwrap(), Color::WHITE);
        assert!(board.color_at_cell(64).is_err());
    }

    #[test]
    fn test_make_move() {
        let mut board = ChessBoard::default();
        assert!(board
            .make_move(
                Pos::H2.into(),
                Pos::H3.into(),
                &mut [64, 64],
                &mut [true, true],
                &mut [true, true]
            )
            .unwrap());
        assert!(!board
            .make_move(
                Pos::H2.into(),
                Pos::H3.into(),
                &mut [64, 64],
                &mut [true, true],
                &mut [true, true]
            )
            .unwrap());
        assert_eq!(board.piece_at_cell(Pos::H2.into()).unwrap(), Piece::NONE);
        assert_eq!(board.piece_at_cell(Pos::H3.into()).unwrap(), Piece::PAWN);
    }
}
