use std::collections::HashMap;
use std::fmt::Display;

use crate::piece::{Piece, PieceType};

pub enum MoveType {
    Normal,
    Castle,
    EnPassant,
    Promotion(PieceType),
}

#[non_exhaustive]
pub struct Move {
    pub start_square: u32,
    pub end_square: u32,
    pub captured_piece: Option<Piece>,
    pub move_type: MoveType,
}

impl Move {
    pub fn new(start_square: u32, end_square: u32, move_type: MoveType) -> Move {
        if start_square > 63 {
            panic!("start square can't be larger than 63");
        }
        if end_square > 63 {
            panic!("end square can't be larger than 63");
        }
        Move {
            start_square,
            end_square,
            captured_piece: None,
            move_type,
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece_chars = HashMap::from([(PieceType::Knight, 'n'), (PieceType::Bishop, 'b'), (PieceType::Rook, 'r'), (PieceType::Queen, 'q')]);
        let files = "abcdefgh";
        let start_file = (self.start_square % 8).to_string();
        let start_rank = (8 - self.start_square / 8).to_string();
        let end_file = (self.end_square % 8).to_string();
        let end_rank = (8 - self.end_square / 8).to_string();
        print!("{}{}{}{}", start_file, start_rank, end_file, end_rank);
        if let MoveType::Promotion(piece) = self.move_type {
            print!("{}", piece_chars.get(&piece).unwrap());
        }
        println!();
        Ok(())
    }
}
