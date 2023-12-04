use crate::board::{piece_move::Move, Board};
use crate::search::{KILLER_MOVE_SLOTS, MAX_DEPTH};
use std::cmp::Ordering;

const HASH_MOVE_SCORE: i32 = 1200;
const CAPTURE_BASE_SCORE: i32 = 1000;
const KILLER_MOVE_SCORE: i32 = 1000;

#[derive(Clone, Copy)]
pub struct OrderingParams {
    pub killer_moves: [[Option<Move>; KILLER_MOVE_SLOTS]; MAX_DEPTH],
}
impl Default for OrderingParams {
    fn default() -> Self {
        Self {
            killer_moves: [[None; KILLER_MOVE_SLOTS]; MAX_DEPTH],
        }
    }
}

pub fn order_moves(board: &Board, moves: &mut [Move], ply: u32, ordering_params: &OrderingParams) {
    moves.sort_by(|a, b| compare_moves(*a, *b, board, ply, ordering_params));
}
fn compare_moves(a: Move, b: Move, board: &Board, ply: u32, ordering_params: &OrderingParams) -> Ordering {
    get_move_score(b, board, ply, ordering_params).cmp(&get_move_score(a, board, ply, ordering_params))
}
fn get_move_score(mov: Move, board: &Board, ply: u32, ordering_params: &OrderingParams) -> i32 {
    let mut score = 0;
    if let Some(captured_piece) = board.squares[mov.to] {
        let piece = board.squares[mov.from].unwrap();
        let capture_score = captured_piece.piece_type().centipawns() - piece.piece_type().centipawns();
        score += CAPTURE_BASE_SCORE;
        score += capture_score;
    }
    if let Some(entry) = board.transposition_table.probe(board.zobrist_hash) {
        if entry.best_move == mov {
            score += HASH_MOVE_SCORE;
        }
    }
    if !board.is_capture(mov) {
        ordering_params.killer_moves[ply as usize].into_iter().for_each(|killer_move_option| {
            if let Some(killer_move) = killer_move_option {
                if mov == killer_move {
                    score += KILLER_MOVE_SCORE;
                }
            }
        });
    }
    score
}
