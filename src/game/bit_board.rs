use serde::Serialize;
use std::{
    cmp::min,
    ops::{BitAnd, BitOr},
};

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash, Serialize)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub fn get_bit(&self, index: u8) -> bool {
        (self.0 & (1 << index)) != 0
    }

    /// Returns the indizes of all 1's
    pub fn get_bits(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        for i in 0..64 {
            if (self.0 & (1 << i)) != 0 {
                result.push(i);
            }
        }

        result
    }

    pub fn set_bit(&mut self, index: u8) {
        self.0 |= 1 << index;
    }

    pub fn clear_bit(&mut self, index: u8) {
        self.0 &= !(1 << index);
    }

    pub fn flip_bit(&mut self, index: u8) {
        self.0 ^= 1 << index;
    }

    pub fn populate_up(&mut self, index: u8, steps: u8, block_mask: BitBoard) {
        let mut current_index = index + 8;
        for _ in 0..steps {
            if current_index >= 64 || block_mask.get_bit(current_index) {
                break;
            }
            self.set_bit(current_index);
            current_index += 8
        }
    }

    pub fn populate_down(&mut self, index: u8, steps: u8, block_mask: BitBoard) {
        let mut current_index = index - 8;
        for _ in 0..steps {
            if current_index >= 64 || block_mask.get_bit(current_index) {
                break;
            }
            self.set_bit(current_index);
            current_index -= 8
        }
    }

    pub fn populate_left(&mut self, index: u8, steps: u8, block_mask: BitBoard) {
        let mut current_index = index - 1;
        let bounded_steps = min(index % 8, steps);
        for _ in 0..bounded_steps {
            if block_mask.get_bit(current_index) {
                break;
            }
            self.set_bit(current_index);

            current_index -= 1
        }
    }

    pub fn populate_right(&mut self, index: u8, steps: u8, block_mask: BitBoard) {
        let mut current_index = index + 1;
        let bounded_steps = min(8 - (index % 8), steps);
        for _ in 0..bounded_steps {
            if block_mask.get_bit(current_index) {
                break;
            }
            self.set_bit(current_index);

            current_index += 1
        }
    }
}

impl From<Vec<u8>> for BitBoard {
    fn from(indices: Vec<u8>) -> Self {
        let bits: u64 = indices
            .into_iter()
            .filter(|&index| index < 64)
            .map(|index| 1 << index)
            .fold(0, |acc, bit| acc | bit);
        BitBoard(bits)
    }
}

impl BitAnd for BitBoard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 & rhs.0)
    }
}

impl BitOr for BitBoard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bit() {
        let board = BitBoard(0xFFFFFFFF00000000);
        assert!(board.get_bit(63));
        assert!(board.get_bit(32));
        assert!(!board.get_bit(31));
        assert!(!board.get_bit(0));
    }

    #[test]
    fn test_set_bit() {
        let mut board = BitBoard(0);
        board.set_bit(0);
        board.set_bit(13);
        assert_eq!(
            board,
            BitBoard(0b0000000000000000000000000000000000000000000000000010000000000001)
        );
    }

    #[test]
    fn test_clear_bit() {
        let mut board = BitBoard(u64::MAX);
        board.clear_bit(0);
        board.clear_bit(13);
        assert_eq!(
            board,
            BitBoard(0b1111111111111111111111111111111111111111111111111101111111111110)
        );
    }

    #[test]
    fn test_flip_bit() {
        let mut board = BitBoard(u64::MAX);
        board.flip_bit(0);
        assert!(!board.get_bit(0));
        board.flip_bit(0);
        assert!(board.get_bit(0));
    }
}
