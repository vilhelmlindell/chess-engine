use std::i8::MAX;
use std::time::Instant;

use crate::transposition_table::{NodeType, TranspositionEntry};
use crate::zobrist_hash::get_zobrist_hash;
use crate::{
    board::Board,
    piece::{Piece, PieceType},
    piece_move::Move,
};

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

        for depth in 1..=MAX_DEPTH {
            let mut highest_eval = i32::MIN;
            let mut curr_best_move: Option<Move> = None;

            let moves = self.generate_moves();

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
                            //return alpha;
                        }
                    }
                    NodeType::UpperBound => {
                        if entry.eval >= beta {
                            return entry.eval;
                            //return beta;
                        }
                    }
                }
            }
        }

        // Depth limit reached
        if depth == 0 {
            let eval = self.quiescence_search(alpha, beta, root_distance);
            //let entry = TranspositionEntry::new(depth, eval, None, NodeType::Exact, self.zobrist_hash);
            //self.transposition_table.store(entry);
            return eval;
        }

        let moves = self.generate_moves();

        // Terminal node
        if moves.is_empty() {
            let king_square = self.piece_squares[Piece::new(PieceType::King, self.side)].lsb();
            let mut eval = 0;
            if self.attacked(king_square) {
                eval = i32::MIN + root_distance as i32;
            }
            //let entry = TranspositionEntry::new(depth, eval, None, NodeType::Exact, self.zobrist_hash);
            //self.transposition_table.store(self.zobrist_hash, depth, eval, NodeType::Exact);
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
        let entry = TranspositionEntry::new(depth, alpha, best_move.unwrap(), NodeType::LowerBound, self.zobrist_hash);
        self.transposition_table.store(entry);
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

        let moves = self.generate_moves();
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

    //pub fn negamax(&mut self, depth: u32, mut alpha: i32, mut beta: i32, root_distance: u32) -> i32 {
    //    let alpha_orig = alpha;

    //    if let Some(entry) = self.transposition_table.probe(self.zobrist_hash) {
    //        if entry.hash == self.zobrist_hash && entry.depth >= depth {
    //            match entry.node_type {
    //                NodeType::Exact => return entry.eval,
    //                NodeType::LowerBound => alpha = alpha.max(entry.eval),
    //                NodeType::UpperBound => beta = beta.min(entry.eval),
    //            }
    //            if alpha >= beta {
    //                return entry.eval;
    //            }
    //        }
    //    }

    //    // Depth limit reached
    //    if depth == 0 {
    //        let eval = self.quiescence_search(alpha, beta, root_distance);
    //        //let eval = self.evaluate();
    //        //self.transposition_table.store(self.zobrist_hash, depth, eval, NodeType::Exact);
    //        return eval;
    //    }

    //    let moves = self.generate_moves();

    //    // Terminal node
    //    if moves.is_empty() {
    //        let king_square = self.piece_squares[Piece::new(PieceType::King, self.side)].lsb();
    //        let mut eval = 0;
    //        if self.attacked(king_square) {
    //            eval = i32::MIN + root_distance as i32;
    //        }
    //        //self.transposition_table.store(self.zobrist_hash, depth, eval, NodeType::Exact);
    //        return eval;
    //    }

    //    let mut best_eval = i32::MIN;
    //    let mut best_move = moves[0];
    //    for mov in moves {
    //        self.make_move(mov);
    //        let eval = -self.negamax(depth - 1, -beta, -alpha, root_distance + 1);
    //        self.unmake_move(mov);

    //        if eval >= best_eval {
    //            best_eval = eval;
    //            best_move = mov;
    //        }
    //        //best_eval = best_eval.max(eval);
    //        alpha = alpha.max(eval);
    //        if alpha >= beta {
    //            break;
    //        }
    //    }

    //    // Transposition Table Store
    //    let node_type = if best_eval <= alpha_orig {
    //        NodeType::UpperBound
    //    } else if best_eval >= beta {
    //        NodeType::LowerBound
    //    } else {
    //        NodeType::Exact
    //    };

    //    let entry = TranspositionEntry::new(depth + 1, best_eval, best_move, node_type, self.zobrist_hash);
    //    //println!("{}", self.zobrist_hash);
    //    //println!("{}", best_move);
    //    self.transposition_table.store(entry);

    //    best_eval
    //}
}
