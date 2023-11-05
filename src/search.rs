pub mod move_ordering;
pub mod transposition_table;

use crate::board::piece::{Piece, PieceType};
use crate::board::piece_move::Move;
use crate::board::Board;
use crate::search::transposition_table::{NodeType, TranspositionEntry};
use std::time::Instant;

const MAX_DEPTH: u32 = 100;

#[derive(Default, Clone, Copy)]
pub struct SearchInfo {
    pub checkmates_found: u32,
    pub transpositions: u32,
    pub positions_evaluated: u32,
}

impl Board {
    pub fn search(&mut self, time: f32) -> Move {
        let start = Instant::now();
        let mut best_move: Option<Move> = None;

        self.transposition_table.clear();

        for depth in 1..=MAX_DEPTH {
            let mut highest_eval = i32::MIN;
            let mut curr_best_move: Option<Move> = None;

            let mut moves = self.generate_moves();
            self.order_moves(&mut moves);

            for mov in moves {
                if start.elapsed().as_secs_f32() > time {
                    if let Some(best_mov) = best_move {
                        println!("Depth: {}", depth - 1);
                        return best_mov;
                    }
                }

                self.make_move(mov);
                let eval = -self.negamax(depth - 1, i32::MIN + 1, i32::MAX, 1);
                self.unmake_move(mov);

                if eval > highest_eval {
                    highest_eval = eval;
                    curr_best_move = Some(mov);
                }
            }

            best_move = curr_best_move;
            let entry = TranspositionEntry::new(0, highest_eval, best_move.unwrap(), NodeType::Exact, self.zobrist_hash);
            self.transposition_table.store(entry);
        }
        best_move.unwrap()
    }

    pub fn negamax(&mut self, depth: u32, mut alpha: i32, beta: i32, root_distance: u32) -> i32 {
        if let Some(entry) = self.transposition_table.probe(self.zobrist_hash) {
            if entry.hash == self.zobrist_hash && entry.depth >= depth {
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
            let eval = self.quiescence_search(alpha, beta, root_distance);
            return eval;
        }

        let mut moves = self.generate_moves();
        self.order_moves(&mut moves);

        // Terminal node
        if moves.is_empty() {
            let king_square = self.piece_squares[Piece::new(PieceType::King, self.side)].lsb();
            let mut eval = 0;
            if self.attacked(king_square) {
                eval = i32::MIN + root_distance as i32;
            }
            return eval;
        }

        let mut best_move: Option<Move> = None;

        for mov in moves {
            self.make_move(mov);
            let eval = -self.negamax(depth - 1, -beta, -alpha, root_distance + 1);
            self.unmake_move(mov);

            if eval >= beta {
                let entry = TranspositionEntry::new(depth, beta, mov, NodeType::UpperBound, self.zobrist_hash);
                self.transposition_table.store(entry);
                return beta;
            }
            if eval >= alpha {
                alpha = eval;
                best_move = Some(mov);
            }
        }

        if let Some(mov) = best_move {
            let entry = TranspositionEntry::new(depth, alpha, mov, NodeType::LowerBound, self.zobrist_hash);
            self.transposition_table.store(entry);
        }
        alpha
    }

    fn quiescence_search(&mut self, mut alpha: i32, beta: i32, root_distance: u32) -> i32 {
        let stand_pat = self.evaluate();
        if stand_pat >= beta {
            return beta;
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let mut moves = self.generate_moves();
        self.order_moves(&mut moves);

        if moves.is_empty() {
            let king_square = self.piece_squares[Piece::new(PieceType::King, self.side)].lsb();
            if self.attacked(king_square) {
                return i32::MIN + root_distance as i32;
            } else {
                return 0;
            }
        }
        let mut num_captures = 0;
        for mov in moves {
            if self.is_capture(mov) {
                num_captures += 1;
                self.make_move(mov);
                let eval = -self.quiescence_search(-beta, -alpha, root_distance + 1);
                self.unmake_move(mov);

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
}
