use crate::board::{piece_move::Move, Board};
use std::cmp::Ordering;
use crate::search::*;

const HASH_MOVE_SCORE: i32 = 1200;
const CAPTURE_BASE_SCORE: i32 = 1000;
const KILLER_MOVE_SCORE: i32 = 1000;

pub fn order_moves(board: &Board, moves: &mut [Move], ply: u32, killer_moves: &[[Option<Move>; KILLER_MOVE_SLOTS]; MAX_DEPTH]) {
    moves.sort_by(|a, b| compare_moves(*a, *b, board, ply, killer_moves));
}
fn compare_moves(a: Move, b: Move, board: &Board, ply: u32, killer_moves: &[[Option<Move>; KILLER_MOVE_SLOTS]; MAX_DEPTH]) -> Ordering {
    get_move_score(b, board, ply, killer_moves).cmp(&get_move_score(a, board, ply, killer_moves))
}
fn get_move_score(mov: Move, board: &Board, ply: u32, killer_moves: &[[Option<Move>; KILLER_MOVE_SLOTS]; MAX_DEPTH]) -> i32 {
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
        for killer_move_option in killer_moves[ply as usize] {
            if let Some(killer_move) = killer_move_option {
                if mov == killer_move {
                    score += KILLER_MOVE_SCORE;
                }
            }
        }
    }
    score
}