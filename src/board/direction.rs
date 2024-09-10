use std::ops::{Index, IndexMut};

use crate::board::Side;
#[derive(Clone, Copy, PartialEq, Eq, Hash)]

pub enum Direction {
    North = -8,
    South = 8,
    West = -1,
    East = 1,
    NorthWest = -9,
    NorthEast = -7,
    SouthWest = 7,
    SouthEast = 9,
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
        [Direction::West, Direction::East, Direction::North, Direction::South]
    }
    pub const fn diagonal() -> [Direction; 4] {
        [Direction::NorthWest, Direction::NorthEast, Direction::SouthWest, Direction::SouthEast]
    }
    pub fn opposite(&self) -> Direction {
        match *self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
            Direction::NorthWest => Direction::SouthEast,
            Direction::NorthEast => Direction::SouthWest,
            Direction::SouthWest => Direction::NorthEast,
            Direction::SouthEast => Direction::NorthWest,
        }
    }
    pub fn up(side: Side) -> Direction {
        match side {
            Side::White => Direction::North,
            Side::Black => Direction::South,
        }
    }
    pub fn down(side: Side) -> Direction {
        match side {
            Side::White => Direction::South,
            Side::Black => Direction::North,
        }
    }
    pub fn from_squares(square1: usize, square2: usize) -> Direction {
        let change = square1 as i32 - square2 as i32;
        if change > 0 {
            if change % 9 == 0 {
                return Direction::NorthWest;
            } else if change % 7 == 0 {
                return Direction::NorthEast;
            } else if change % 8 == 0 {
                return Direction::North;
            }
        } else if change % 9 == 0 {
            return Direction::SouthEast;
        } else if change % 7 == 0 {
            return Direction::SouthWest;
        } else if change % 8 == 0 {
            return Direction::South;
        }
        let rank1 = square1 / 8;
        let rank2 = square2 / 8;
        if rank1 == rank2 {
            if square1 < square2 {
                return Direction::West;
            } else {
                return Direction::East;
            }
        }
        Direction::North
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
