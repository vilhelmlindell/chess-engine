use bitflags::bitflags;

use crate::board::piece::PieceType;
use crate::board::{square_from_string, Board};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum MoveType {
    #[default]
    Normal,
    Castle {
        kingside: bool,
    },
    DoublePush,
    EnPassant,
    Promotion(PieceType),
}

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct Move {
    pub from: usize,
    pub to: usize,
    pub move_type: MoveType,
}

impl Move {
    pub fn new(start_square: usize, end_square: usize, move_type: MoveType) -> Move {
        if start_square > 63 {
            panic!("start square can't be larger than 63");
        }
        if end_square > 63 {
            panic!("end square can't be larger than 63");
        }

        Move {
            from: start_square,
            to: end_square,
            move_type,
        }
    }
    //pub fn from() -> {}
    //pub fn to() -> {}
    //pub fn move_type() -> MoveType {}
    pub fn from_long_algebraic_notation(string: &str, board: &Board) -> Move {
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
                move_type = MoveType::Castle { kingside: true };
            } else if start_file - end_file == 2 {
                move_type = MoveType::Castle { kingside: false };
            }
        }

        Move::new(start_square, end_square, move_type)
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece_chars = HashMap::from([(PieceType::Knight, 'n'), (PieceType::Bishop, 'b'), (PieceType::Rook, 'r'), (PieceType::Queen, 'q')]);
        let files = "abcdefgh";
        let start_file = files.chars().nth(self.from % 8).unwrap();
        let start_rank = (8 - self.from / 8).to_string();
        let end_file = files.chars().nth(self.to % 8).unwrap();
        let end_rank = (8 - self.to / 8).to_string();
        write!(f, "{}{}{}{}", start_file, start_rank, end_file, end_rank).unwrap();
        if let MoveType::Promotion(piece) = self.move_type {
            write!(f, "{}", piece_chars.get(&piece).unwrap()).unwrap();
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
        let from_order = self.from.cmp(&other.from);
        let to_order = self.to.cmp(&other.to);

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
