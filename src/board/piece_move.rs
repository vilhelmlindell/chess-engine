use num_enum::UnsafeFromPrimitive;

use crate::board::piece::PieceType;
use crate::board::{square_from_string, Board};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum MoveType {
    Normal,
    KingsideCastle,
    QueensideCastle,
    DoublePush,
    EnPassant,
    RookPromotion,
    BishopPromotion,
    QueenPromotion,
    KnightPromotion,
}

impl MoveType {
    pub const PROMOTIONS: [MoveType; 4] = [MoveType::BishopPromotion, MoveType::RookPromotion, MoveType::QueenPromotion, MoveType::KnightPromotion];

    pub fn promotion_piece(&self) -> PieceType {
        match self {
            MoveType::KnightPromotion => PieceType::Knight,
            MoveType::BishopPromotion => PieceType::Bishop,
            MoveType::QueenPromotion => PieceType::Queen,
            MoveType::RookPromotion => PieceType::Rook,
            _ => panic!("Enum variant is not a promotion"),
        }
    }
}

pub type Square = usize;

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct Move {
    bits: u16,
}

impl Move {
    pub fn new(start_square: usize, end_square: usize, move_type: MoveType) -> Move {
        //assert!(start_square < 64, "start square can't be larger than 63");
        //assert!(end_square < 64, "end square can't be larger than 63");

        Move {
            bits: ((move_type as u16) << 12) | ((end_square as u16) << 6) | (start_square as u16),
        }
    }

    pub fn from(&self) -> Square {
        (self.bits & 0b111111) as Square
    }
    pub fn to(&self) -> Square {
        ((self.bits & 0b111111000000) >> 6) as Square
    }
    pub fn move_type(&self) -> MoveType {
        unsafe { MoveType::unchecked_transmute_from(((self.bits & 0b1111000000000000) >> 12) as u8) }
    }
    pub fn from_long_algebraic(string: &str, board: &Board) -> Move {
        let start_square = square_from_string(&string[0..2]);
        let end_square = square_from_string(&string[2..4]);

        let start_rank = (start_square % 8) as i32;
        let start_file = (start_square / 8) as i32;

        let end_rank = (end_square % 8) as i32;
        let end_file = (end_square / 8) as i32;

        let piece_type = board.squares[start_square].unwrap().piece_type();
        let mut move_type = MoveType::Normal;

        if piece_type == PieceType::Pawn {
            if i32::abs(start_rank - end_rank) == 2 {
                move_type = MoveType::DoublePush;
            } else if i32::abs(start_file - end_file) == 1 && board.squares[end_square].is_none() {
                move_type = MoveType::EnPassant;
            }
        }

        if piece_type == PieceType::King {
            if start_file - end_file == -2 {
                move_type = MoveType::KingsideCastle;
            } else if start_file - end_file == 2 {
                move_type = MoveType::QueensideCastle;
            }
        }

        Move::new(start_square, end_square, move_type)
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece_chars = HashMap::from([(PieceType::Knight, 'n'), (PieceType::Bishop, 'b'), (PieceType::Rook, 'r'), (PieceType::Queen, 'q')]);
        let files = "abcdefgh";
        let start_file = files.chars().nth(self.from() % 8).unwrap();
        let start_rank = (8 - self.from() / 8).to_string();
        let end_file = files.chars().nth(self.to() % 8).unwrap();
        let end_rank = (8 - self.to() / 8).to_string();
        write!(f, "{}{}{}{}", start_file, start_rank, end_file, end_rank).unwrap();
        if MoveType::PROMOTIONS.contains(&self.move_type()) {
            let piece_type = self.move_type().promotion_piece();
            write!(f, "{}", piece_chars.get(&piece_type).unwrap()).unwrap();
        }
        Ok(())
    }
}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Move {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let from_order = self.from().cmp(&other.from());
        let to_order = self.to().cmp(&other.to());

        from_order.then(to_order)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_to_long_algebraic_notation() {
        let mov = Move::new(0, 4, MoveType::Normal);
        assert_eq!(mov.to_string(), "a8e8");
    }
}
