use crate::board::Side;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Piece {
    WhitePawn = 0,
    WhiteBishop,
    WhiteKnight,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackBishop,
    BlackKnight,
    BlackRook,
    BlackQueen,
    BlackKing,
}

impl Piece {
    pub fn new(piece_type: &PieceType, side: &Side) -> Piece {
        match piece_type {
            PieceType::Pawn => match &side {
                Side::White => Piece::WhitePawn,
                Side::Black => Piece::BlackPawn,
            },
            PieceType::Bishop => match &side {
                Side::White => Piece::WhiteBishop,
                Side::Black => Piece::BlackBishop,
            },
            PieceType::Knight => match &side {
                Side::White => Piece::WhiteKnight,
                Side::Black => Piece::BlackKnight,
            },
            PieceType::Rook => match &side {
                Side::White => Piece::WhiteRook,
                Side::Black => Piece::BlackRook,
            },
            PieceType::Queen => match &side {
                Side::White => Piece::WhiteQueen,
                Side::Black => Piece::BlackQueen,
            },
            PieceType::King => match &side {
                Side::White => Piece::WhiteKing,
                Side::Black => Piece::BlackKing,
            },
        }
    }
    pub fn piece_type(&self) -> PieceType {
        match self {
            Piece::WhitePawn | Piece::BlackPawn => PieceType::Pawn,
            Piece::WhiteKnight | Piece::BlackKnight => PieceType::Knight,
            Piece::WhiteBishop | Piece::BlackBishop => PieceType::Bishop,
            Piece::WhiteRook | Piece::BlackRook => PieceType::Rook,
            Piece::WhiteQueen | Piece::BlackQueen => PieceType::Queen,
            Piece::WhiteKing | Piece::BlackKing => PieceType::King,
        }
    }
    pub fn side(&self) -> Side {
        match self {
            Piece::WhitePawn | Piece::WhiteKnight | Piece::WhiteBishop | Piece::WhiteRook | Piece::WhiteQueen | Piece::WhiteKing => Side::White,
            Piece::BlackPawn | Piece::BlackKnight | Piece::BlackBishop | Piece::BlackRook | Piece::BlackQueen | Piece::BlackKing => Side::Black,
        }
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PieceType {
    Pawn = 0,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
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
