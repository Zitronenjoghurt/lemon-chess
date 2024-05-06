use crate::game::error::GameError;
use serde::Serialize;
use std::ops::{BitAnd, BitOr};

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash, Serialize)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub fn validate_index(index: u8) -> Result<(), GameError> {
        if index >= 64 {
            return Err(GameError::ValidationError(
                "Board address out of range.".to_string(),
            ));
        }
        Ok(())
    }

    pub fn get_bit(&self, index: u8) -> Result<bool, GameError> {
        Self::validate_index(index)?;
        Ok((self.0 & (1 << index)) != 0)
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

    pub fn set_bit(&mut self, index: u8) -> Result<(), GameError> {
        Self::validate_index(index)?;
        self.0 |= 1 << index;
        Ok(())
    }

    pub fn clear_bit(&mut self, index: u8) -> Result<(), GameError> {
        Self::validate_index(index)?;
        self.0 &= !(1 << index);
        Ok(())
    }

    pub fn flip_bit(&mut self, index: u8) -> Result<(), GameError> {
        Self::validate_index(index)?;
        self.0 ^= 1 << index;
        Ok(())
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
    fn test_invalid_index_error() {
        let mut board = BitBoard(0);
        assert!(board.get_bit(64).is_err());
        assert!(board.set_bit(64).is_err());
        assert!(board.clear_bit(64).is_err());
        assert!(board.flip_bit(64).is_err());
    }

    #[test]
    fn test_get_bit() {
        let board = BitBoard(0xFFFFFFFF00000000);
        assert!(board.get_bit(63).unwrap());
        assert!(board.get_bit(32).unwrap());
        assert!(!board.get_bit(31).unwrap());
        assert!(!board.get_bit(0).unwrap());
    }

    #[test]
    fn test_set_bit() {
        let mut board = BitBoard(0);
        board.set_bit(0).unwrap();
        board.set_bit(13).unwrap();
        assert_eq!(
            board,
            BitBoard(0b0000000000000000000000000000000000000000000000000010000000000001)
        );
    }

    #[test]
    fn test_clear_bit() {
        let mut board = BitBoard(u64::MAX);
        board.clear_bit(0).unwrap();
        board.clear_bit(13).unwrap();
        assert_eq!(
            board,
            BitBoard(0b1111111111111111111111111111111111111111111111111101111111111110)
        );
    }

    #[test]
    fn test_flip_bit() {
        let mut board = BitBoard(u64::MAX);
        board.flip_bit(0).unwrap();
        assert!(!board.get_bit(0).unwrap());
        board.flip_bit(0).unwrap();
        assert!(board.get_bit(0).unwrap());
    }
}
