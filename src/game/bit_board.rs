#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub fn get_bit(&self, index: u64) -> bool {
        (self.0 & (1 << index)) != 0
    }

    pub fn set_bit(&mut self, index: u64) {
        self.0 |= 1 << index;
    }

    pub fn clear_bit(&mut self, index: u64) {
        self.0 &= !(1 << index);
    }

    pub fn flip_bit(&mut self, index: u64) {
        self.0 ^= 1 << index;
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
        let mut board = BitBoard(0x0000000000000000);
        board.set_bit(0);
        board.set_bit(13);
        assert_eq!(
            board,
            BitBoard(0b0000000000000000000000000000000000000000000000000010000000000001)
        );
    }

    #[test]
    fn test_clear_bit() {
        let mut board = BitBoard(0xFFFFFFFFFFFFFFFF);
        board.clear_bit(0);
        board.clear_bit(13);
        assert_eq!(
            board,
            BitBoard(0b1111111111111111111111111111111111111111111111111101111111111110)
        );
    }

    #[test]
    fn test_flip_bit() {
        let mut board = BitBoard(0xFFFFFFFFFFFFFFFF);
        board.flip_bit(0);
        assert!(!board.get_bit(0));
        board.flip_bit(0);
        assert!(board.get_bit(0));
    }
}
