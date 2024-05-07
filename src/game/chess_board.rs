use crate::game::{bit_board::BitBoard, color::Color, error::GameError, piece::Piece};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize)]
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
    pub fn validate_index(index: u8) -> Result<(), GameError> {
        if index >= 64 {
            return Err(GameError::ValidationError(
                "Board address out of range.".to_string(),
            ));
        }
        Ok(())
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
        let occupied_by_black = self.colors[0].get_bit(index);
        let occupied_by_white = self.colors[1].get_bit(index);
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

    pub fn mask_by_piece_and_color(&self, piece: Piece, color: Color) -> BitBoard {
        self.pieces[piece as usize] & self.colors[color as usize]
    }

    pub fn make_move(&mut self, from: u8, to: u8) -> Result<bool, GameError> {
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

        Ok(true)
    }

    pub fn generate_legal_moves(
        &self,
        color: Color,
        initial_pawn_mask: BitBoard,
    ) -> Result<AvailableMoves, GameError> {
        let piece_indices = self.colors[color as usize].get_bits();
        let mut piece_moves: Vec<(u8, Vec<u8>)> = Vec::new();

        for index in piece_indices {
            let piece = self.piece_at_cell(index)?;
            let action_mask = piece.get_action_mask(
                index,
                color.opponent_color(),
                initial_pawn_mask,
                self.colors,
            );

            let target_indices = action_mask.get_bits();
            let mut valid_targets: Vec<u8> = Vec::new();
            for target_index in target_indices {
                if !Self::does_move_lead_to_check(self, color, index, target_index) {
                    valid_targets.push(target_index)
                }
            }
            piece_moves.push((index, valid_targets))
        }

        Ok(AvailableMoves(piece_moves))
    }

    /// If a move leads to your own king being in check
    pub fn does_move_lead_to_check(&self, color: Color, from: u8, to: u8) -> bool {
        let mut future_board = self.clone();
        if let Ok(success) = future_board.make_move(from, to) {
            if success {
                return future_board.is_king_check(color);
            }
        }
        false
    }

    pub fn get_king_check_positions(&self, color: Color) -> Vec<u8> {
        let king_board = Self::mask_by_piece_and_color(self, Piece::KING, color);
        let king_positions = king_board.get_bits();
        if king_positions.is_empty() {
            return Vec::new();
        }
        let king_index = king_positions[0];
        let block_mask = self.colors[0] | self.colors[1];
        let threat_masks = Piece::get_king_threat_masks(king_index, block_mask);

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
        assert!(board.make_move(Pos::H2.into(), Pos::H3.into()).unwrap());
        assert!(!board.make_move(Pos::H2.into(), Pos::H3.into()).unwrap());
        assert_eq!(board.piece_at_cell(Pos::H2.into()).unwrap(), Piece::NONE);
        assert_eq!(board.piece_at_cell(Pos::H3.into()).unwrap(), Piece::PAWN);
    }
}
