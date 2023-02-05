use crate::piece::Piece;

pub enum MoveType {
    Normal,
    Castle,
    EnPassant,
    Promotion,
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
