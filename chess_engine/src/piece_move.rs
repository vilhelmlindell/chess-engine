use std::collections::HashMap;
use std::fmt::Display;

use crate::piece::{Piece, PieceType};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MoveType {
    Normal,
    Castle { kingside: bool },
    DoublePush,
    EnPassant,
    Promotion(PieceType),
}

#[non_exhaustive]
#[derive(PartialEq, Eq)]
pub struct Move {
    pub start_square: usize,
    pub end_square: usize,
    pub move_type: MoveType,
}

impl Move {
    pub fn new(start_square: &usize, end_square: &usize, move_type: &MoveType) -> Move {
        if *start_square > 63 {
            panic!("start square can't be larger than 63");
        }
        if *end_square > 63 {
            panic!("end square can't be larger than 63");
        }
        Move {
            start_square: *start_square,
            end_square: *end_square,
            move_type: *move_type,
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece_chars = HashMap::from([
            (PieceType::Knight, 'n'),
            (PieceType::Bishop, 'b'),
            (PieceType::Rook, 'r'),
            (PieceType::Queen, 'q'),
        ]);
        let files = "abcdefgh";
        let start_file = files.chars().nth(self.start_square % 8).unwrap();
        let start_rank = (8 - self.start_square / 8).to_string();
        let end_file = files.chars().nth(self.end_square % 8).unwrap();
        let end_rank = (8 - self.end_square / 8).to_string();
        write!(f, "{}{}{}{}", start_file, start_rank, end_file, end_rank).unwrap();
        if let MoveType::Promotion(piece) = self.move_type {
            write!(f, "{}", piece_chars.get(&piece).unwrap()).unwrap();
        }
        Ok(())
    }
}

//impl PartialOrd for Move {
//    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//
//    }
//}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_to_long_algebraic_notation() {
        let mov = Move::new(&0, &4, &MoveType::Normal);
        assert_eq!(mov.to_string(), "a8e8");
    }
}
