use crate::board::piece::{Piece, PieceType};
use crate::board::piece_move::Move;
use crate::board::Board;
use crate::search::transposition_table::{NodeType, TranspositionEntry};
use std::time::Instant;
use crate::evaluation::evaluate;
use crate::move_generation::generate_moves;
use crate::search::move_ordering::order_moves;
use super::book_moves::get_book_move;

pub const MAX_DEPTH: usize = 100;
pub const KILLER_MOVE_SLOTS: usize = 3;

#[derive(Default, Clone, Copy)]
pub struct SearchResult {
    pub checkmates_found: u32,
    pub transpositions: u32,
    pub positions_evaluated: u32,
}

struct SearchState {
    killer_moves: [[Option<Move>; KILLER_MOVE_SLOTS]; MAX_DEPTH],
    diagnostics: SearchDiagnostics
}

struct SearchDiagnostics {
    positions_evaluated: u32,
    transpositions: u32,
}

pub fn search(board: &mut Board, time: f32) -> Move {
    if let Some(book_move) = get_book_move(board, 0.5) {
        return book_move;
    }

    let start = Instant::now();
    let mut best_move: Option<Move> = None;

    board.transposition_table.clear();
    
    let mut killer_moves = [[None; KILLER_MOVE_SLOTS]; MAX_DEPTH];

    for depth in 1..=MAX_DEPTH as u32 {
        let mut highest_eval = i32::MIN;
        let mut curr_best_move: Option<Move> = None;

        let mut moves = generate_moves(board);
        order_moves(board, &mut moves, 0, &killer_moves);

        for mov in moves {
            if start.elapsed().as_secs_f32() > time {
                if let Some(best_mov) = best_move {
                    return best_mov;
                }
            }

            board.make_move(mov);
            let eval = -negamax(board, depth - 1, i32::MIN + 1, i32::MAX, 0, &mut killer_moves);
            board.unmake_move(mov);

            if eval > highest_eval {
                highest_eval = eval;
                curr_best_move = Some(mov);
            }
        }

        best_move = curr_best_move;
        let entry = TranspositionEntry::new(0, highest_eval, best_move.unwrap(), NodeType::Exact, board.zobrist_hash);
        board.transposition_table.store(entry);
    }
    best_move.unwrap()
}

pub fn negamax(board: &mut Board, depth: u32, mut alpha: i32, beta: i32, ply: u32) -> i32 {
    if let Some(entry) = board.transposition_table.probe(board.zobrist_hash) {
        if entry.hash == board.zobrist_hash && entry.depth >= depth {
            match entry.node_type {
                NodeType::Exact => return entry.eval,
                NodeType::LowerBound => {
                    if entry.eval <= alpha {
                        return entry.eval;
                    }
                }
                NodeType::UpperBound => {
                    if entry.eval >= beta {
                        return entry.eval;
                    }
                }
            }
        }
    }

    // Depth limit reached
    if depth == 0 {
        let eval = quiescence_search(board, alpha, beta, ply + 1);
        return eval;
    }

    let mut moves = generate_moves(board);
    order_moves(board, &mut moves, ply);

    // Terminal node
    if moves.is_empty() {
        let king_square = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();
        let mut eval = 0;
        if board.attacked(king_square) {
            eval = i32::MIN + ply as i32;
        }
        return eval;
    }

    let mut best_move: Option<Move> = None;

    for mov in moves {
        board.make_move(mov);
        let eval = -negamax(board, depth - 1, -beta, -alpha, ply + 1, killer_moves);
        board.unmake_move(mov);

        if eval >= beta {
            let entry = TranspositionEntry::new(depth, beta, mov, NodeType::UpperBound, board.zobrist_hash);
            board.transposition_table.store(entry);
            if board.squares[mov.to].is_none() {
                for i in (KILLER_MOVE_SLOTS - 2)..=0 {
                    killer_moves[ply as usize][i + 1] = killer_moves[ply as usize][i];
                    killer_moves[ply as usize][0] = Some(mov);
                }
            }
            return beta;
        }
        if eval >= alpha {
            alpha = eval;
            best_move = Some(mov);
        }
    }

    if let Some(mov) = best_move {
        let entry = TranspositionEntry::new(depth, alpha, mov, NodeType::LowerBound, board.zobrist_hash);
        board.transposition_table.store(entry);
    }
    alpha
}

fn quiescence_search(board: &mut Board, mut alpha: i32, beta: i32, ply_from_root: u32) -> i32 {
    let stand_pat = evaluate(board);
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let mut moves = generate_moves(board);
    order_moves(board, &mut moves, ply, killer_moves);

    if moves.is_empty() {
        let king_square = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();
        return if board.attacked(king_square) {
            i32::MIN + ply_from_root as i32
        } else {
            0
        }
    }
    let mut num_captures = 0;
    for mov in moves {
        if board.is_capture(mov) {
            num_captures += 1;
            board.make_move(mov);
            let eval = -quiescence_search(board, -beta, -alpha, ply + 1, killer_moves);
            board.unmake_move(mov);

            if eval >= beta {
                return beta;
            }
            if eval > alpha {
                alpha = eval;
            }
        }
    }
    if num_captures == 0 {
        return stand_pat;
    }
    if let Some(mov) = best_move {
        let entry = TranspositionEntry::new(depth, alpha, mov, NodeType::LowerBound, board.zobrist_hash);
        board.transposition_table.store(entry);
    }
    alpha
}