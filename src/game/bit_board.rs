use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    fmt::{self, Display, Formatter},
    ops::{Add, BitAnd, BitOr, Not},
};

// Will be De/Serialized as a Bitstring to avoid having too large numbers for bson to handle
#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub fn get_bit(&self, index: u8) -> bool {
        if index >= 64 {
            return false;
        }
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
        if index >= 64 {
            return;
        }
        self.0 |= 1 << index;
    }

    pub fn clear_bit(&mut self, index: u8) {
        if index >= 64 {
            return;
        }
        self.0 &= !(1 << index);
    }

    pub fn flip_bit(&mut self, index: u8) {
        if index >= 64 {
            return;
        }
        self.0 ^= 1 << index;
    }

    pub fn populate_up(&mut self, index: u8, steps: u8, block_mask: BitBoard) {
        let mut current_index = index + 8;
        for _ in 0..steps {
            if current_index >= 64 {
                break;
            }
            self.set_bit(current_index);
            if block_mask.get_bit(current_index) {
                break;
            }
            current_index += 8;
        }
    }

    pub fn populate_down(&mut self, index: u8, steps: u8, block_mask: BitBoard) {
        let mut current_index = index.wrapping_sub(8);
        for _ in 0..steps {
            if current_index >= 64 {
                break;
            }
            self.set_bit(current_index);
            if block_mask.get_bit(current_index) {
                break;
            }
            current_index = current_index.wrapping_sub(8);
        }
    }

    pub fn populate_left(&mut self, index: u8, steps: u8, block_mask: BitBoard) {
        let mut current_index = index.wrapping_sub(1);
        for _ in 0..steps {
            self.set_bit(current_index);
            if block_mask.get_bit(current_index) || current_index % 8 == 0 {
                break;
            }
            current_index = current_index.wrapping_sub(1);
        }
    }

    pub fn populate_right(&mut self, index: u8, steps: u8, block_mask: BitBoard) {
        let mut current_index = index + 1;
        for _ in 0..steps {
            self.set_bit(current_index);
            if block_mask.get_bit(current_index) || current_index % 8 == 7 {
                break;
            }
            current_index += 1;
        }
    }

    pub fn populate_up_right(&mut self, index: u8, steps: u8, block_mask: BitBoard) {
        let mut current_index = index + 9;
        for _ in 0..steps {
            if current_index >= 64 {
                break;
            }
            self.set_bit(current_index);
            if block_mask.get_bit(current_index) || current_index % 8 == 7 {
                break;
            }
            current_index += 9;
        }
    }

    pub fn populate_up_left(&mut self, index: u8, steps: u8, block_mask: BitBoard) {
        let mut current_index = index + 7;
        for _ in 0..steps {
            if current_index >= 64 {
                break;
            }
            self.set_bit(current_index);
            if block_mask.get_bit(current_index) || current_index % 8 == 0 {
                break;
            }
            current_index += 7;
        }
    }

    pub fn populate_down_right(&mut self, index: u8, steps: u8, block_mask: BitBoard) {
        let mut current_index = index.wrapping_sub(7);
        for _ in 0..steps {
            if current_index >= 64 {
                break;
            }
            self.set_bit(current_index);
            if block_mask.get_bit(current_index) || current_index % 8 == 7 {
                break;
            }
            current_index = current_index.wrapping_sub(7);
        }
    }

    pub fn populate_down_left(&mut self, index: u8, steps: u8, block_mask: BitBoard) {
        let mut current_index = index.wrapping_sub(9);
        for _ in 0..steps {
            if current_index >= 64 {
                break;
            }
            self.set_bit(current_index);
            if block_mask.get_bit(current_index) || current_index % 8 == 0 {
                break;
            }
            current_index = current_index.wrapping_sub(9);
        }
    }

    pub fn populate_vert_hor(&mut self, index: u8, steps: u8, block_mask: BitBoard) {
        self.populate_up(index, steps, block_mask);
        self.populate_down(index, steps, block_mask);
        self.populate_left(index, steps, block_mask);
        self.populate_right(index, steps, block_mask);
    }

    pub fn populate_diag(&mut self, index: u8, steps: u8, block_mask: BitBoard) {
        self.populate_up_right(index, steps, block_mask);
        self.populate_down_right(index, steps, block_mask);
        self.populate_down_left(index, steps, block_mask);
        self.populate_up_left(index, steps, block_mask);
    }

    pub fn populate_jump(&mut self, index: u8, row: i8, column: i8) {
        let current_row = (index / 8) as i8;
        let current_col = (index % 8) as i8;

        let new_row = current_row + row;
        let new_col = current_col + column;

        if !(0..=7).contains(&new_row) || !(0..=7).contains(&new_col) {
            return;
        }

        let new_index = ((new_row * 8) + new_col) as u8;
        self.set_bit(new_index);
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

impl Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        BitBoard(!self.0)
    }
}

impl Add<u8> for BitBoard {
    type Output = BitBoard;

    fn add(self, index: u8) -> Self::Output {
        let mut board = self;
        board.set_bit(index);
        board
    }
}

impl Display for BitBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut grid = String::new();
        for row in (0..8).rev() {
            for col in 0..8 {
                let index = row * 8 + col;
                if self.get_bit(index as u8) {
                    grid.push('1');
                } else {
                    grid.push('0');
                }
                if col < 7 {
                    grid.push(' ');
                }
            }
            grid.push('\n');
        }
        write!(f, "{}", grid)
    }
}

impl Serialize for BitBoard {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:064b}", self.0))
    }
}

struct BitBoardVisitor;

impl<'de> Visitor<'de> for BitBoardVisitor {
    type Value = BitBoard;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a binary string of length 64")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value.len() != 64 {
            return Err(E::custom(format!(
                "expected a binary string of length 64, got length {}",
                value.len()
            )));
        }

        let mut bits = 0;
        for (i, char) in value.chars().enumerate() {
            if char == '1' {
                bits |= 1 << (63 - i); // Assuming big endian bit order
            } else if char != '0' {
                return Err(E::invalid_value(de::Unexpected::Char(char), &self));
            }
        }
        Ok(BitBoard(bits))
    }
}

impl<'de> Deserialize<'de> for BitBoard {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(BitBoardVisitor)
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
