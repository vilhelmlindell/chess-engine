use crate::board::direction::Direction;
use derive_more::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, MulAssign, Not, Shl, ShlAssign, Shr, ShrAssign};
use std::fmt::Display;
use std::ops::Deref;

const NOT_A_FILE: u64 = 0xfefefefefefefefe;
const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;

#[derive(MulAssign, ShrAssign, ShlAssign, BitOrAssign, BitAndAssign, BitXorAssign, BitAnd, BitOr, BitXor, Shr, Shl, Not, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub fn from_square(square: usize) -> Self {
        Self(1 << square)
    }

    pub fn bit(&self, n: usize) -> u64 {
        (self.0 >> n) & 1
    }
    pub fn set_bit(&mut self, n: usize) {
        self.0 |= 1 << n;
    }
    pub fn clear_bit(&mut self, n: usize) {
        self.0 &= !(1 << n);
    }
    pub fn lsb(&self) -> usize {
        self.trailing_zeros() as usize
    }
    pub fn msb(&self) -> usize {
        63 - self.leading_zeros() as usize
    }
    pub fn pop_lsb(&mut self) -> usize {
        let index = self.lsb();
        self.clear_bit(index);
        index
    }
    //pub fn pop_msb(&mut self) -> usize {
    //    let index = self.msb();
    //    self.clear_bit(index);
    //    index
    //}

    pub const fn north(&self) -> Bitboard {
        Bitboard(self.0 >> 8)
    }
    pub const fn south(&self) -> Bitboard {
        Bitboard(self.0 << 8)
    }
    pub const fn west(&self) -> Bitboard {
        Bitboard((self.0 >> 1) & NOT_H_FILE)
    }
    pub const fn east(&self) -> Bitboard {
        Bitboard((self.0 << 1) & NOT_A_FILE)
    }
    pub const fn north_west(&self) -> Bitboard {
        Bitboard((self.0 >> 9) & NOT_H_FILE)
    }
    pub const fn north_east(&self) -> Bitboard {
        Bitboard((self.0 >> 7) & NOT_A_FILE)
    }
    pub const fn south_west(&self) -> Bitboard {
        Bitboard((self.0 << 7) & NOT_H_FILE)
    }
    pub const fn south_east(&self) -> Bitboard {
        Bitboard((self.0 << 9) & NOT_A_FILE)
    }
    pub const fn shift(&self, direction: Direction) -> Bitboard {
        match direction {
            Direction::North => self.north(),
            Direction::South => self.south(),
            Direction::West => self.west(),
            Direction::East => self.east(),
            Direction::NorthWest => self.north_west(),
            Direction::NorthEast => self.north_east(),
            Direction::SouthWest => self.south_west(),
            Direction::SouthEast => self.south_east(),
        }
        //const DIRECTIONS: [fn(&self) -> Bitboard; 8] = [
        //    Self::north, Self::south, Self::west, Self::east,
        //    Self::north_west, Self::north_east, Self::south_west, Self::south_east
        //];

        //DIRECTIONS[direction as usize](self)
    }
}
impl Iterator for Bitboard {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }
        let index = self.pop_lsb();
        Some(index)
    }
}
impl Deref for Bitboard {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in 0..8 {
            for file in 0..8 {
                write!(f, " {}", self.bit(rank * 8 + file)).unwrap();
            }
            writeln!(f).unwrap();
        }
        Ok(())
    }
}
impl PartialEq<u64> for Bitboard {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shifts_bits_correctly() {
        let bitboard = Bitboard(0x00FF000000000000);
        assert_eq!(bitboard.shift(Direction::NorthEast), Bitboard(0x0000fe0000000000));
    }
}
