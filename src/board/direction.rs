use std::ops::{Index, IndexMut};
use num_enum::{UnsafeFromPrimitive};

use crate::board::Side;
#[derive(Clone, Copy, PartialEq, Eq, Hash, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum Direction {
    North,
    West,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
    East,
    South,
}

const DIRECTION_VALUES: [i32; 8] = [-8, -1, -9, -7, 7, 9, 1, 8];

impl Direction {
    pub const fn value(self) -> i32 {
        DIRECTION_VALUES[self as usize]
    }
    pub const fn all() -> [Direction; 8] {
        [
            Direction::North,
            Direction::West,
            Direction::NorthWest,
            Direction::NorthEast,
            Direction::SouthWest,
            Direction::SouthEast,
            Direction::East,
            Direction::South,
        ]
    }
    pub const fn orthogonal() -> [Direction; 4] {
        [Direction::West, Direction::East, Direction::North, Direction::South]
    }
    pub const fn diagonal() -> [Direction; 4] {
        [Direction::NorthWest, Direction::NorthEast, Direction::SouthWest, Direction::SouthEast]
    }
    pub fn opposite(self) -> Direction {
        unsafe { UnsafeFromPrimitive::unchecked_transmute_from(7 - self as u8) }
    }
    pub fn up(side: Side) -> Direction {
        unsafe { UnsafeFromPrimitive::unchecked_transmute_from(7 * side as u8) }
    }
    pub fn down(side: Side) -> Direction {
        unsafe { UnsafeFromPrimitive::unchecked_transmute_from(7 * (1 - side as u8)) }
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
