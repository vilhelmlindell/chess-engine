use std::ops::{Index, IndexMut};
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
