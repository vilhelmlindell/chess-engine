use crate::board::piece::{Piece, PieceType};
use crate::board::piece_move::Move;
use crate::board::Board;
use crate::evaluation::evaluate;
use crate::move_generation::generate_moves;
use crate::search::transposition_table::{NodeType, TranspositionEntry};
use std::cmp::Ordering;
use std::time::Instant;

use super::book_moves::get_book_move;

pub const MAX_DEPTH: usize = 100;
pub const KILLER_MOVE_SLOTS: usize = 3;

const HASH_MOVE_SCORE: i32 = 1200;
const CAPTURE_BASE_SCORE: i32 = 1000;
const KILLER_MOVE_SCORE: i32 = 2000;

const MAX_EVAL: i32 = 100000000;

pub struct Search {
    pub result: SearchResult,
    pub max_time: f32,
    pub killer_moves: [[Option<Move>; KILLER_MOVE_SLOTS]; MAX_DEPTH],
    pub pv: Vec<Move>,
    start_time: Instant,
}

#[derive(Default, Clone)]
pub struct SearchResult {
    //pub best_move: Option<Move>,
    pub pv: Vec<Move>,
    pub highest_eval: i32,
    pub depth_reached: u32,
    pub nodes: u32,
    pub transpositions: u32,
    pub time: u128,
}

impl Search {
    pub fn new(max_time: f32) -> Self {
        Self {
            result: SearchResult::default(),
            start_time: Instant::now(),
            max_time,
            killer_moves: [[None; KILLER_MOVE_SLOTS]; MAX_DEPTH],
            pv: Vec::with_capacity(MAX_DEPTH),
        }
    }

    pub fn extract_pv(&mut self, board: &mut Board) {
        self.pv.clear();
        let mut current_hash = board.zobrist_hash;

        while let Some(entry) = board.transposition_table.probe(current_hash) {
            if entry.hash != current_hash || self.pv.len() >= MAX_DEPTH {
                break;
            }

            let pv_move = entry.best_move;
            self.pv.push(pv_move);

            // Make the move on the board to get the next position
            board.make_move(pv_move);
            current_hash = board.zobrist_hash;
        }

        // Unmake all the moves to restore the original board state
        for &mov in self.pv.iter().rev() {
            board.unmake_move(mov);
        }
    }

    pub fn search(&mut self, board: &mut Board) -> SearchResult {
        board.transposition_table.clear();
        self.pv.clear();
        self.start_time = Instant::now();
        self.killer_moves = [[None; KILLER_MOVE_SLOTS]; MAX_DEPTH];

        // Initialize search state
        self.result = SearchResult::default();

        if let Some(book_move) = get_book_move(board, 1.0) {
            self.result.pv.push(book_move);
            return self.result.clone();
        }

        // Iterate over increasing depths
        for depth in 1..=MAX_DEPTH as u32 {
            let alpha = -MAX_EVAL;
            let beta = MAX_EVAL;

            let eval = self.negamax(board, depth, alpha, beta, 0);

            if self.start_time.elapsed().as_secs_f32() > self.max_time {
                break;
            }

            self.result.highest_eval = eval;
            self.result.depth_reached = depth;

            self.extract_pv(board);
        }

        self.result.pv = self.pv.clone();
        self.result.time = self.start_time.elapsed().as_millis();
        self.result.clone()
    }
    
    fn negamax(&mut self, board: &mut Board, depth: u32, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        self.result.nodes += 1;

        if self.start_time.elapsed().as_secs_f32() > self.max_time {
            return alpha;
        }

        if let Some(entry) = board.transposition_table.probe(board.zobrist_hash) {
            if entry.hash == board.zobrist_hash && entry.depth >= depth {
                self.result.transpositions += 1;
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
            return self.quiescence_search(board, alpha, beta, ply + 1);
        }

        let mut moves = generate_moves(board);
        self.order_moves(board, &mut moves, ply);

        // Terminal node
        if moves.is_empty() {
            let king_square = board.piece_squares[Piece::new(PieceType::King, board.side) as usize].lsb();
            return if board.attacked(king_square) {
                -MAX_EVAL + ply as i32
            } else {
                0 // Stalemate
            };
        }

        let mut best_move: Option<Move> = None;
        let mut evaluation_bound = NodeType::UpperBound;

        for mov in moves {
            board.make_move(mov);
            let eval = -self.negamax(board, depth - 1, -beta, -alpha, ply + 1);
            board.unmake_move(mov);

            // Move was *too* good, opponent will choose a different move earlier on to avoid this position.
            // (Beta-cutoff / Fail high)
            if eval >= beta {
                let entry = TranspositionEntry::new(depth, beta, mov, NodeType::LowerBound, board.zobrist_hash);
                board.transposition_table.store(entry);
                if !board.is_capture(mov) {
                    self.update_killer_moves(mov, ply);
                }
                return beta;
            }
            // Found a new best move in this position
            if eval > alpha {
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

    fn quiescence_search(&mut self, board: &mut Board, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        self.result.nodes += 1;

        let stand_pat = evaluate(board);
        if stand_pat >= beta {
            return beta;
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let moves = generate_moves(board);
        if moves.is_empty() {
            let king_square = board.piece_squares[Piece::new(PieceType::King, board.side) as usize].lsb();
            return if board.attacked(king_square) {
                -MAX_EVAL + ply as i32
            } else {
                0 // Stalemate
            };
        }

        for mov in moves {
            if board.is_capture(mov) {
                board.make_move(mov);
                let eval = -self.quiescence_search(board, -beta, -alpha, ply + 1);
                board.unmake_move(mov);

                if eval >= beta {
                    return beta;
                }
                if eval > alpha {
                    alpha = eval;
                }
            }
        }
        alpha
    }

    pub fn order_moves(&self, board: &Board, moves: &mut [Move], ply: u32) {
        moves.sort_by(|a, b| self.compare_moves(*a, *b, board, ply));
    }

    fn compare_moves(&self, a: Move, b: Move, board: &Board, ply: u32) -> Ordering {
        self.get_move_score(b, board, ply).cmp(&self.get_move_score(a, board, ply))
    }

    fn get_move_score(&self, mov: Move, board: &Board, ply: u32) -> i32 {
        if let Some(pv_mov) = self.pv.get(ply as usize) {
            if mov == *pv_mov {
                return i32::MAX;
            }
        }

        let mut score = 0;
        if board.is_capture(mov) {
            let captured_piece = board.squares[mov.to()].unwrap();
            let moving_piece = board.squares[mov.from()].unwrap();
            let capture_score = captured_piece.piece_type().centipawns() - moving_piece.piece_type().centipawns();
            score += CAPTURE_BASE_SCORE + capture_score;
        }
        if let Some(entry) = board.transposition_table.probe(board.zobrist_hash) {
            if entry.best_move == mov {
                score += HASH_MOVE_SCORE;
            }
        }
        if !board.is_capture(mov) {
            for killer_move in &self.killer_moves[ply as usize] {
                if let Some(killer) = killer_move {
                    if mov == *killer {
                        score += KILLER_MOVE_SCORE;
                        break;
                    }
                }
            }
        }
        score
    }

    fn update_killer_moves(&mut self, mov: Move, ply: u32) {
        let ply = ply as usize;
        if !self.killer_moves[ply].contains(&Some(mov)) {
            self.killer_moves[ply].rotate_right(1);
            self.killer_moves[ply][0] = Some(mov);
        }
    }
}
