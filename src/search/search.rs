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
    pub highest_eval: i32,
    pub depth_reached: u32,
    pub nodes: u32,
    pub transpositions: u32,
    pub time: f32,
}

const MAX_EVAL: i32 = 100000000;

pub fn search(max_time: f32, board: &mut Board) -> SearchResult {
    board.transposition_table.clear();

    let mut search_state = SearchState::default();

    if let Some(book_move) = get_book_move(board, 1.0) {
        search_state.result.best_move = Some(book_move);
        return search_state.result;
    }

    let start = Instant::now();

    search_state.result.highest_eval = -MAX_EVAL;

    for depth in 1..=MAX_DEPTH as u32 {
        let mut moves = generate_moves(board);
        order_moves(board, &mut moves, 0, &search_state.ordering_params);

        for mov in moves {
            let time_searched = start.elapsed().as_secs_f32();
            if time_searched > max_time {
                search_state.result.depth_reached = (depth as i32 - 1) as u32;
                search_state.result.time = time_searched;
                return search_state.result;
            }

            board.make_move(mov);
            let eval = -negamax(board, depth - 1, -MAX_EVAL, MAX_EVAL, 0, &mut search_state);
            board.unmake_move(mov);

            //println!("Move: {}, Eval: {}", mov, eval);
            //println!("Highest eval: {}", highest_eval);
            //println!("Best move: {}", search_state.result.best_move.unwrap);
            if eval > search_state.result.highest_eval {
                search_state.result.highest_eval = eval;
                search_state.result.best_move = Some(mov);
            }
        }
    }
    //search_state.result.depth_reached = (MAX_DEPTH - 1) as u32;
    search_state.result
}

pub fn negamax(board: &mut Board, depth: u32, mut alpha: i32, beta: i32, ply: u32, state: &mut SearchState) -> i32 {
    state.result.nodes += 1;
    if let Some(entry) = board.transposition_table.probe(board.zobrist_hash) {
        if entry.hash == board.zobrist_hash && entry.depth >= depth {
            state.result.transpositions += 1;
            match entry.node_type {
                NodeType::Exact => {
                    return entry.eval;
                }
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
        let eval = quiescence_search(board, alpha, beta, ply + 1, state);
        return eval;
    }

    let mut moves = generate_moves(board);
    order_moves(board, &mut moves, ply, &state.ordering_params);

    // Terminal node
    if moves.is_empty() {
        let king_square = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();
        return if board.attacked(king_square) { -MAX_EVAL + ply as i32 } else { 0 };
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
            if board.squares[mov.to()].is_none() {
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
    state.result.nodes += 1;
    let stand_pat = evaluate(board);
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let moves = generate_moves(board);

    if moves.is_empty() {
        let king_square = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();
        return if board.attacked(king_square) { -MAX_EVAL + ply as i32 } else { 0 };
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
