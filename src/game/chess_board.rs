use crate::game::{bit_board::BitBoard, color::Color, error::GameError, piece::Piece};
use base64::{engine::general_purpose::STANDARD, Engine as _};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ChessBoard {
    colors: [BitBoard; 2],
    pieces: [BitBoard; 6],
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

impl ChessBoard {
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
        let occupied_by_black = self.colors[0].get_bit(index)?;
        let occupied_by_white = self.colors[1].get_bit(index)?;
        Ok(occupied_by_black || occupied_by_white)
    }

    pub fn piece_at_cell(&self, index: u8) -> Result<Piece, GameError> {
        for piece_id in 0..6 {
            if self.pieces[piece_id].get_bit(index)? {
                return Ok(Piece::from(piece_id));
            }
        }

        Ok(Piece::NONE)
    }

    pub fn color_at_cell(&self, index: u8) -> Result<Color, GameError> {
        for color_id in 0..2 {
            if self.colors[color_id].get_bit(index)? {
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

    pub fn make_move(&mut self, from: u8, to: u8) -> Result<bool, GameError> {
        let piece = Self::piece_at_cell(self, from)?;
        if piece == Piece::NONE {
            return Ok(false);
        }

        // Update piece
        let piece_index = piece as usize;
        self.pieces[piece_index].clear_bit(from)?;
        self.pieces[piece_index].set_bit(to)?;

        // Update color
        if self.colors[0].get_bit(from)? {
            self.colors[0].clear_bit(from)?;
            self.colors[0].set_bit(to)?;
        } else {
            self.colors[1].clear_bit(from)?;
            self.colors[1].set_bit(to)?;
        }

        Ok(true)
    }

    /*
    pub fn generate_legal_moves(&self, color: Color) -> Result<Vec<(u8, u8)>, GameError> {
        let piece_indizes = self.colors[color as usize].get_bits();
    }
    */
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
