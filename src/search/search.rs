use crate::board::piece::{Piece, PieceType};
use crate::board::piece_move::Move;
use crate::board::Board;
use crate::evaluation::evaluate;
use crate::move_generation::generate_moves;
use crate::search::book_moves::get_book_move;
use crate::search::move_ordering::order_moves;
use crate::search::move_ordering::OrderingParams;
use crate::search::transposition_table::{NodeType, TranspositionEntry};
use std::time::Instant;

pub const MAX_DEPTH: usize = 100;
pub const KILLER_MOVE_SLOTS: usize = 3;

#[derive(Default, Clone, Copy)]
pub struct SearchState {
    pub ordering_params: OrderingParams,
    pub result: SearchResult,
}

#[derive(Default, Clone, Copy)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub depth_reached: u32,
    pub positions_evaluated: u32,
    pub transpositions: u32,
}

pub fn search(time: f32, board: &mut Board) -> SearchResult {
    board.transposition_table.clear();

    let mut search_state = SearchState::default();

    if let Some(book_move) = get_book_move(board, 0.5) {
        search_state.result.best_move = Some(book_move);
        return search_state.result;
    }

    let start = Instant::now();

    for depth in 1..=MAX_DEPTH as u32 {
        let mut highest_eval = i32::MIN;

        let mut moves = generate_moves(board);
        order_moves(board, &mut moves, 0, &search_state.ordering_params);

        for mov in moves {
            if start.elapsed().as_secs_f32() > time {
                search_state.result.depth_reached = (depth as i32 - 1) as u32;
                return search_state.result;
            }

            board.make_move(mov);
            let eval = -negamax(board, depth - 1, i32::MIN + 1, i32::MAX, 0, &mut search_state);
            board.unmake_move(mov);

            if eval > highest_eval {
                highest_eval = eval;
                search_state.result.best_move = Some(mov);
            }
        }
    }
    search_state.result.depth_reached = (MAX_DEPTH - 1) as u32;
    search_state.result
}

pub fn negamax(board: &mut Board, depth: u32, mut alpha: i32, beta: i32, ply: u32, state: &mut SearchState) -> i32 {
    if let Some(entry) = board.transposition_table.probe(board.zobrist_hash) {
        if entry.hash == board.zobrist_hash && entry.depth >= depth {
            state.result.transpositions += 1;
            match entry.node_type {
                NodeType::Exact => {
                    state.result.positions_evaluated += 1;
                    return entry.eval;
                }
                NodeType::LowerBound => {
                    if entry.eval <= alpha {
                        state.result.positions_evaluated += 1;
                        return entry.eval;
                    }
                }
                NodeType::UpperBound => {
                    if entry.eval >= beta {
                        state.result.positions_evaluated += 1;
                        return entry.eval;
                    }
                }
            }
        }
    }

    // Depth limit reached
    if depth == 0 {
        let eval = quiescence_search(board, alpha, beta, ply + 1, state);
        state.result.positions_evaluated += 1;
        return eval;
    }

    let mut moves = generate_moves(board);
    order_moves(board, &mut moves, ply, &state.ordering_params);

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
    let mut evaluation_bound = NodeType::UpperBound;

    for mov in moves {
        board.make_move(mov);
        let eval = -negamax(board, depth - 1, -beta, -alpha, ply + 1, state);
        board.unmake_move(mov);

        if eval >= beta {
            let entry = TranspositionEntry::new(depth, beta, mov, NodeType::LowerBound, board.zobrist_hash);
            board.transposition_table.store(entry);
            if board.squares[mov.to].is_none() {
                state.ordering_params.killer_moves[ply as usize].rotate_right(1);
                state.ordering_params.killer_moves[ply as usize][0] = Some(mov);
            }
            return beta;
        }
        if eval >= alpha {
            alpha = eval;
            evaluation_bound = NodeType::Exact;
            best_move = Some(mov);
        }
    }

    if let Some(mov) = best_move {
        let entry = TranspositionEntry::new(depth, alpha, mov, evaluation_bound, board.zobrist_hash);
        board.transposition_table.store(entry);
    }
    alpha
}

fn quiescence_search(board: &mut Board, mut alpha: i32, beta: i32, ply: u32, state: &mut SearchState) -> i32 {
    let stand_pat = evaluate(board);
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let mut moves = generate_moves(board);

    if moves.is_empty() {
        let king_square = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();
        return if board.attacked(king_square) { i32::MIN + ply as i32 } else { 0 };
    }
    //order_moves(board, &mut moves, ply, &state.ordering_params);
    let mut num_captures = 0;
    for mov in moves {
        if board.is_capture(mov) {
            num_captures += 1;
            board.make_move(mov);
            let eval = -quiescence_search(board, -beta, -alpha, ply + 1, state);
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
    alpha
}
