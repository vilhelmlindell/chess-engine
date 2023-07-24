use crate::{board::Board, piece_move::Move};
use crate::{piece, piece_move::*};
use std::cmp::Ordering;

impl Board {
    pub fn compare_moves(&self, a: Move, b: Move) -> Ordering {
        let is_a_capture = self.is_capture(a);
        let is_b_capture = self.is_capture(b);

        match (is_a_capture, is_b_capture) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => Ordering::Equal, // TODO: implement sorting non captures
            (true, true) => self.compare_captures(a, b),
        }
    }
    fn compare_captures(&self, a: Move, b: Move) -> Ordering {
        let piece_a = self.squares[a.from].unwrap() as u8;
        let piece_b = self.squares[b.from].unwrap() as u8;
        let captured_piece_a = self.squares[a.to].unwrap() as u8;
        let captured_piece_b = self.squares[b.to].unwrap() as u8;

        captured_piece_b.cmp(&captured_piece_a).then_with(|| piece_a.cmp(&piece_b))
    }
}
