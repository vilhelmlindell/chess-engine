use arrayvec::ArrayVec;

use crate::{
    board::{piece::PieceType, piece_move::Move, Board},
    move_generation::{MoveVec, MAX_LEGAL_MOVES},
};

use super::{search, Search, KILLER_MOVE_SLOTS, MAX_DEPTH, USE_KILLER};

pub struct MoveOrderer {
    pub moves: MoveVec,
    pub move_scores: [u32; MAX_LEGAL_MOVES],
    pub start: usize,
}

impl MoveOrderer {
    pub fn new(moves: MoveVec, search: &Search, hash_move: Option<Move>, ply: u32) -> Self {
        let mut move_scores = [0; MAX_LEGAL_MOVES];
        Self { moves, move_scores, start: 0 }
    }
    pub fn next<const SHOULD_SCORE: bool>(&mut self, search: &Search, hash_move: Option<Move>, ply: u32) -> Option<Move> {
        if self.start >= self.moves.len() {
            return None;
        }

        let mut highest_score = 0;
        let mut best_i = self.start;
        for i in self.start..self.moves.len() {
            if SHOULD_SCORE {
                self.move_scores[i] = move_score(self.moves[i], search, hash_move, ply)
            }
            if self.move_scores[i] > highest_score {
                best_i = i;
                highest_score = self.move_scores[i];
            }
        }

        let mov = self.moves[best_i];
        self.moves.swap(self.start, best_i);
        self.move_scores.swap(self.start, best_i);
        self.start += 1;
        return Some(mov);
    }
}

pub enum MoveCategory {
    QuietMove,
    LosingCapture,
    EqualCapture,
    KillerMove,
    WinningCapture,
    Promotion,
    HashMove,
    PvMove,
}

impl MoveCategory {
    pub fn base_score(self) -> u32 {
        (self as u32) << 24
    }
}

fn move_score(mov: Move, search: &Search, hash_move: Option<Move>, ply: u32) -> u32 {
    if Some(mov) == search.result.pv.get(ply as usize).cloned() {
        return MoveCategory::PvMove.base_score();
    }

    if Some(mov) == hash_move {
        return MoveCategory::HashMove.base_score();
    }

    if search.board.is_capture(mov) {
        let captured_piece = search.board.squares[mov.to()].unwrap();
        let moving_piece = search.board.squares[mov.from()].unwrap();
        let capture_score = captured_piece.piece_type().centipawns() - moving_piece.piece_type().centipawns();

        if capture_score > 0 {
            return (MoveCategory::WinningCapture.base_score() as i32 + capture_score) as u32;
        } else if capture_score == 0 {
            return MoveCategory::EqualCapture.base_score();
        } else {
            return (MoveCategory::LosingCapture.base_score() as i32 + capture_score) as u32;
        }
    }

    if let Some(piece) = mov.move_type().promotion_piece() {
        let promotion_score = (piece.centipawns() - PieceType::Pawn.centipawns()) as u32;
        return MoveCategory::Promotion.base_score() + promotion_score;
    }

    if USE_KILLER && search.is_killer(mov, ply) {
        return MoveCategory::KillerMove.base_score();
    }

    let history_score = search.history[search.board.side][mov.from()][mov.to()];
    MoveCategory::QuietMove.base_score() + history_score
}

pub mod test {
    use std::collections::HashSet;

    use crate::{board, move_generation::generate_moves};

    use super::*;
    #[test]
    fn move_orderer_iterator() {
        let search = Search {
            board: Board::from_fen("rnb1kbnr/ppp1pppp/8/3q4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1"),
            ..Default::default()
        };
        let moves = generate_moves(&search.board);
        let mut iterated_moves = Vec::new();
        let mut move_orderer = MoveOrderer::new(moves.clone(), &search, None, 0);
        for mov in move_orderer {
            iterated_moves.push(mov);
            println!("mov: {}", mov);
        }
        let m1: HashSet<_> = moves.iter().cloned().collect();
        let m2: HashSet<_> = iterated_moves.iter().cloned().collect();
        assert_eq!(m1.symmetric_difference(&m2).count(), 0);
    }
}
