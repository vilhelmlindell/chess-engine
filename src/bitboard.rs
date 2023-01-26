use derive_more::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, MulAssign, Not, Shl, ShlAssign,
    Shr, ShrAssign,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;
use std::ops::Index;
use std::ops::IndexMut;

const NOT_A_FILE: u64 = 0xfefefefefefefefe;
const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North = 0,
    South,
    West,
    East,
    SouthWest,
    NorthWest,
    NorthEast,
    SouthEast,
}

impl Direction {
    pub const fn value(&self) -> i32 {
        match *self {
            Direction::North => -8,
            Direction::South => 8,
            Direction::West => -1,
            Direction::East => 1,
            Direction::NorthWest => -9,
            Direction::NorthEast => -7,
            Direction::SouthWest => 7,
            Direction::SouthEast => 9,
        }
    }
    pub const fn index(&self) -> usize {
        match *self {
            Direction::North => 0,
            Direction::South => 1,
            Direction::West => 2,
            Direction::East => 3,
            Direction::NorthWest => 4,
            Direction::NorthEast => 5,
            Direction::SouthWest => 6,
            Direction::SouthEast => 7,
        }
    }
    pub const fn all() -> [Direction; 8] {
        [
            Direction::West,
            Direction::East,
            Direction::North,
            Direction::South,
            Direction::NorthWest,
            Direction::NorthEast,
            Direction::SouthWest,
            Direction::SouthEast,
        ]
    }
    pub const fn orthagonal() -> [Direction; 4] {
        [
            Direction::West,
            Direction::East,
            Direction::North,
            Direction::South,
        ]
    }
    pub const fn diagonal() -> [Direction; 4] {
        [
            Direction::NorthWest,
            Direction::NorthEast,
            Direction::SouthWest,
            Direction::SouthEast,
        ]
    }
}

impl<T, const N: usize> Index<Direction> for [T; N] {
    type Output = T;

    fn index(&self, index: Direction) -> &Self::Output {
        &self[index as usize]
    }
}
impl<T, const N: usize> IndexMut<Direction> for [T; N] {
    fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

#[derive(
    MulAssign,
    ShrAssign,
    ShlAssign,
    BitOrAssign,
    BitAndAssign,
    BitXorAssign,
    BitAnd,
    BitOr,
    BitXor,
    Shr,
    Shl,
    Not,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
)]
pub struct Bitboard(pub u64);

impl Bitboard {
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
    pub const fn shift(&self, direction: &Direction) -> Bitboard {
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
    }
    pub fn get_bit(&self, n: &u32) -> u64 {
        (self.0 >> n) & 1
    }
    pub fn set_bit(&mut self, n: &u32) {
        self.0 |= 1 << n;
    }
    pub fn clear_bit(&mut self, n: &u32) {
        self.0 &= !(1 << n);
    }
    pub fn pop_lsb(&mut self) -> u32 {
        let index = self.trailing_zeros();
        self.clear_bit(&index);
        index
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
                write!(f, " {}", self.get_bit(&(rank * 8 + file)).to_string()).unwrap();
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
    use crate::board::Board;

    use super::*;
    #[test]
    fn shifts_bits_correctly() {
        let bitboard = Bitboard(0x00FF000000000000);
        assert_eq!(
            bitboard.shift(&Direction::NorthEast),
            Bitboard(0x0000fe0000000000)
        );
    }
}
