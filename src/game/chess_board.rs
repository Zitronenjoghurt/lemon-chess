use crate::game::{bit_board::BitBoard, enums::Piece};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ChessBoard {
    color: BitBoard,
    pieces: [BitBoard; 6],
}

// Black will always be at the bottom and white at the top of the board
// Representation can later be individualized by rotating the board
// Index starts in the bottom left
impl Default for ChessBoard {
    fn default() -> Self {
        Self {
            color: BitBoard(0b1111111111111111000000000000000000000000000000000000000000000000),
            pieces: [
                BitBoard(0b0000000011111111000000000000000000000000000000001111111100000000),
                BitBoard(0b0010010000000000000000000000000000000000000000000000000000100100),
                BitBoard(0b0100001000000000000000000000000000000000000000000000000001000010),
                BitBoard(0b1000000100000000000000000000000000000000000000000000000010000001),
                BitBoard(0b0001000000000000000000000000000000000000000000000000000000010000),
                BitBoard(0b0000100000000000000000000000000000000000000000000000000000001000),
            ],
        }
    }
}

impl ChessBoard {
    pub fn make_move(&mut self, piece: Piece, from: u64, to: u64) {
        let piece_index = piece as usize;

        // Update piece
        self.pieces[piece_index].clear_bit(from);
        self.pieces[piece_index].set_bit(to);

        // Update color
        if self.color.get_bit(from) {
            self.color.clear_bit(from);
            self.color.set_bit(to);
        }
    }
}
