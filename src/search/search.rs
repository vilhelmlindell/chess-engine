use crate::board::piece::{Piece, PieceType};
use crate::board::piece_move::Move;
use crate::board::Board;
use crate::evaluation::evaluate;
use crate::move_generation::generate_moves;
use crate::search::transposition_table::{NodeType, TranspositionEntry};
use std::cmp::Ordering;
use std::thread;
use std::time::Instant;

use super::book_moves::get_book_move;

pub const MAX_DEPTH: usize = 100;
pub const KILLER_MOVE_SLOTS: usize = 3;

const HASH_MOVE_SCORE: i32 = 1200;
const CAPTURE_BASE_SCORE: i32 = 1000;
const KILLER_MOVE_SCORE: i32 = 2000;

const MAX_EVAL: i32 = 100000000;

pub struct Search {
    pub args: SearchArgs,
    pub result: SearchResult,
    pub max_time: u32,
    pub killer_moves: [[Option<Move>; KILLER_MOVE_SLOTS]; MAX_DEPTH],
    pub pv: Vec<Move>,
    start_time: Instant,
}

#[derive(Default, Clone, Copy)]
pub struct SearchArgs {
    pub time_left: Option<[u32; 2]>, // None here signifies infinite time
    pub time_increment: [u32; 2],
}

#[derive(Default, Clone)]
pub struct SearchResult {
    //pub best_move: Option<Move>,
    pub pv: Vec<Move>,
    pub highest_eval: i32,
    pub depth_reached: u32,
    pub nodes: u32,
    pub transpositions: u32,
    pub time: u32,
}

impl Default for Search {
    fn default() -> Self {
        Self {
            args: SearchArgs::default(),
            result: SearchResult::default(),
            start_time: Instant::now(),
            max_time: 0,
            killer_moves: [[None; KILLER_MOVE_SLOTS]; MAX_DEPTH],
            pv: Vec::with_capacity(MAX_DEPTH),
        }
    }
}

impl Search {
    pub fn search(&mut self, search_args: SearchArgs, board: &mut Board) -> SearchResult {
        self.args = search_args;
        self.pv.clear();
        self.result = SearchResult::default();
        self.killer_moves = [[None; KILLER_MOVE_SLOTS]; MAX_DEPTH];
        self.start_time = Instant::now();
        self.max_time = self.calculate_time(board);
        println!("{}", self.max_time);

        //let vote_map = [0; 64 * 64];
        //let available_threads: usize = thread::available_parallelism().unwrap().into();
        //let mut threads = Vec::with_capacity(available_threads);

        //for i in 0..available_threads {
        //    threads.push(thread::spawn(move || {}));
        //}

        board.transposition_table.clear();

        if let Some(book_move) = get_book_move(board, 1.0) {
            self.result.pv.push(book_move);
            return self.result.clone();
        }

        for depth in 1..=MAX_DEPTH as u32 {
            let alpha = -MAX_EVAL;
            let beta = MAX_EVAL;

            let eval = self.pvs(board, depth, alpha, beta, 0);

            self.result.highest_eval = eval;
            self.result.depth_reached = depth;
            self.result.pv = self.extract_pv(kj);
            self.result.time = self.start_time.elapsed().as_millis() as u32;

            self.extract_pv(board);
            Search::print_info(&self.result);

            if self.start_time.elapsed().as_millis() as u32 > self.max_time {
                Search::print_info(&self.result);
            }
        }

        self.result.clone()
    }

    fn print_info(result: &SearchResult) {
        print!(
            "info depth {} score cp {} time {} nodes {} nps {} ",
            result.depth_reached, result.highest_eval, result.time, result.nodes, result.nodes
        );
    }

    fn calculate_time(&mut self, board: &Board) -> u32 {
        let half_moves_left = Search::remaining_half_moves(board.total_material);
        let time_left = self.args.time_left[board.side] + self.args.time_increment[board.side] * half_moves_left / 2;
        time_left / half_moves_left
    }
    // Approximation for amount of half moves remaining
    // See: http://facta.junis.ni.ac.rs/acar/acar200901/acar2009-07.pdf for more info
    fn remaining_half_moves(material: u32) -> u32 {
        match material {
            0..20 => material + 10,
            20..=60 => 3 * material / 8 + 22,
            61.. => 5 * material / 4 - 30,
        }
    }

    fn pvs(&mut self, board: &mut Board, depth: u32, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        if self.start_time.elapsed().as_millis() as u32 > self.max_time {
            return alpha;
        }

        self.result.nodes += 1;

        // Transposition table lookup
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

        // Leaf node evaluation
        if depth == 0 {
            return self.quiescence_search(board, alpha, beta, ply + 1);
        }

        let mut moves = generate_moves(board);
        self.order_moves(board, &mut moves, ply);

        // Check for terminal positions
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

        let first_move = moves[0];
        board.make_move(first_move);
        let score = -self.pvs(board, depth - 1, -beta, -alpha, ply + 1);
        board.unmake_move(first_move);

        if score > alpha {
            if score >= beta {
                let entry = TranspositionEntry::new(depth, beta, first_move, NodeType::LowerBound, board.zobrist_hash);
                board.transposition_table.store(entry);
                if !board.is_capture(first_move) {
                    self.update_killer_moves(first_move, ply);
                }
                return beta;
            }
            evaluation_bound = NodeType::Exact;
            best_move = Some(first_move);
            alpha = score;
        }

        // Search remaining moves with zero window
        for mov in moves.into_iter().skip(1) {
            if alpha >= beta {
                break;
            }

            board.make_move(mov);

            // Try null-window search first
            let mut score = -self.pvs(board, depth - 1, -alpha - 1, -alpha, ply + 1);

            // If the null-window search failed high, do a full re-search
            if score > alpha && score < beta {
                score = -self.pvs(board, depth - 1, -beta, -alpha, ply + 1);
            }

            board.unmake_move(mov);

            if score >= beta { // Move is too good, the opponent has better option
                let entry = TranspositionEntry::new(depth, beta, mov, NodeType::LowerBound, board.zobrist_hash);
                board.transposition_table.store(entry);
                if !board.is_capture(mov) {
                    self.update_killer_moves(mov, ply);
                }
                return beta;
            }

            if score > alpha { // Move is within 
                evaluation_bound = NodeType::Exact;
                best_move = Some(mov);
                alpha = score;
            }
        }

        // Store position in transposition table
        if let Some(mov) = best_move {
            let entry = TranspositionEntry::new(depth, alpha, mov, evaluation_bound, board.zobrist_hash);
            board.transposition_table.store(entry);
        }

        alpha
    }

    fn quiescence_search(&mut self, board: &mut Board, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        if self.start_time.elapsed().as_millis() as u32 > self.max_time {
            return alpha;
        }

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

    pub fn extract_pv(&mut self, board: &mut Board) -> Vec<Move> {
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
