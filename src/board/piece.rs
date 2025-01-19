use num_enum::UnsafeFromPrimitive;

use crate::board::Side;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum Piece {
    WhitePawn = Side::White as u8 | (PieceType::Pawn as u8) << 1,
    WhiteKnight = Side::White as u8 | (PieceType::Knight as u8) << 1,
    WhiteBishop = Side::White as u8 | (PieceType::Bishop as u8) << 1,
    WhiteRook = Side::White as u8 | (PieceType::Rook as u8) << 1,
    WhiteQueen = Side::White as u8 | (PieceType::Queen as u8) << 1,
    WhiteKing = Side::White as u8 | (PieceType::King as u8) << 1,
    BlackPawn = Side::Black as u8 | (PieceType::Pawn as u8) << 1,
    BlackKnight = Side::Black as u8 | (PieceType::Knight as u8) << 1,
    BlackBishop = Side::Black as u8 | (PieceType::Bishop as u8) << 1,
    BlackRook = Side::Black as u8 | (PieceType::Rook as u8) << 1,
    BlackQueen = Side::Black as u8 | (PieceType::Queen as u8) << 1,
    BlackKing = Side::Black as u8 | (PieceType::King as u8) << 1,
}

impl Piece {
    const PIECE_MASK: u8 = 0b1110;
    const SIDE_MASK: u8 = 0b0001;

    pub fn new(piece_type: PieceType, side: Side) -> Piece {
        unsafe { Piece::unchecked_transmute_from(((piece_type as u8) << 1) | (side as u8)) }
    }
    pub fn piece_type(&self) -> PieceType {
        unsafe { PieceType::unchecked_transmute_from(((*self as u8) & Piece::PIECE_MASK) >> 1) }
    }
    pub fn side(&self) -> Side {
        unsafe { Side::unchecked_transmute_from((*self as u8) & Piece::SIDE_MASK) }
    }
    pub fn all() -> [Piece; 12] {
        [
            Piece::WhitePawn,
            Piece::WhiteBishop,
            Piece::WhiteKnight,
            Piece::WhiteRook,
            Piece::WhiteQueen,
            Piece::WhiteKing,
            Piece::BlackPawn,
            Piece::BlackBishop,
            Piece::BlackKnight,
            Piece::BlackRook,
            Piece::BlackQueen,
            Piece::BlackKing,
        ]
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum PieceType {
    Pawn = 0b000,
    Knight = 0b001,
    Bishop = 0b010,
    Rook = 0b011,
    Queen = 0b100,
    King = 0b101,
}

impl PieceType {
    pub fn centipawns(&self) -> i32 {
        match self {
            PieceType::Pawn => 100,
            PieceType::Knight => 320,
            PieceType::Bishop => 330,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => 20000,
        }
    }
    pub const fn standard_value(&self) -> u32 {
        match self {
            PieceType::Pawn => 1,
            PieceType::Knight => 3,
            PieceType::Bishop => 3,
            PieceType::Rook => 5,
            PieceType::Queen => 9,
            PieceType::King => 0,
        }
    }
    pub const fn phase(&self) -> i32 {
        match self {
            PieceType::Pawn => 1,
            PieceType::Knight => 2,
            PieceType::Bishop => 2,
            PieceType::Rook => 3,
            PieceType::Queen => 5,
            PieceType::King => 0,
        }
    }
    pub fn is_slider(&self) -> bool {
        matches!(self, PieceType::Bishop | PieceType::Rook | PieceType::Queen)
    }
}

impl<T, const N: usize> Index<Piece> for [T; N] {
    type Output = T;

    fn index(&self, index: Piece) -> &Self::Output {
        &self[index as usize]
    }
}
impl<T, const N: usize> IndexMut<Piece> for [T; N] {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl<T, const N: usize> Index<PieceType> for [T; N] {
    type Output = T;

    fn index(&self, index: PieceType) -> &Self::Output {
        &self[index as usize]
    }
}
impl<T, const N: usize> IndexMut<PieceType> for [T; N] {
    fn index_mut(&mut self, index: PieceType) -> &mut Self::Output {
        &mut self[index as usize]
    }
}
