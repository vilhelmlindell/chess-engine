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

            let mut moves = self.generate_moves();

            //if let Some(first_move) = best_move {
            //    moves.insert(0, first_move);
            //}
            //self.principal_variation(depth);

            for mov in moves {
                if start.elapsed().as_secs_f32() > time {
                    if let Some(best_mov) = best_move {
                        //println!("Depth: {}", depth - 1);
                        //println!("Transpositions: {}", search_info.transpositions);
                        return best_mov;
                    }
                }
                if depth == 1 && best_move.is_some() {
                    continue;
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
            let entry = TranspositionEntry::new(1, highest_eval, best_move.unwrap(), NodeType::Exact, get_zobrist_hash(self));
            self.transposition_table.store(entry);
        }
        best_move.unwrap()
    }
    pub fn negamax(&mut self, depth: u32, mut alpha: i32, mut beta: i32, root_distance: u32) -> i32 {
        let alpha_orig = alpha;

        if let Some(entry) = self.transposition_table.probe(get_zobrist_hash(self)) {
            if entry.hash == get_zobrist_hash(self) && entry.depth >= depth {
                match entry.node_type {
                    NodeType::Exact => return entry.eval,
                    NodeType::LowerBound => alpha = alpha.max(entry.eval),
                    NodeType::UpperBound => beta = beta.min(entry.eval),
                }
                if alpha >= beta {
                    return entry.eval;
                }
            }
        }

        // Depth limit reached
        if depth == 0 {
            let eval = self.quiescence_search(alpha, beta, root_distance);
            //let eval = self.evaluate();
            //self.transposition_table.store(get_zobrist_hash(self), depth, eval, NodeType::Exact);
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
            //self.transposition_table.store(get_zobrist_hash(self), depth, eval, NodeType::Exact);
            return eval;
        }

        let mut best_eval = i32::MIN;
        let mut best_move = moves[0];
        for mov in moves {
            self.make_move(mov);
            let eval = -self.negamax(depth - 1, -beta, -alpha, root_distance + 1);
            self.unmake_move(mov);

            if eval >= best_eval {
                best_eval = eval;
                best_move = mov;
            }
            //best_eval = best_eval.max(eval);
            alpha = alpha.max(eval);
            if alpha >= beta {
                break;
            }
        }

        // Transposition Table Store
        let node_type = if best_eval <= alpha_orig {
            NodeType::UpperBound
        } else if best_eval >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };

        let entry = TranspositionEntry::new(depth, best_eval, best_move, node_type, get_zobrist_hash(self));
        //println!("{}", get_zobrist_hash(self));
        //println!("{}", best_move);
        self.transposition_table.store(entry);

        best_eval
    }

    //pub fn negamax(&mut self, depth: u32, mut alpha: i32, beta: i32, root_distance: u32) -> i32 {
    //    if let Some(entry) = self.transposition_table.probe(get_zobrist_hash(self)) {
    //        if entry.hash == get_zobrist_hash(self) && entry.depth >= depth {
    //            match entry.node_type {
    //                NodeType::Exact => return entry.eval,
    //                NodeType::LowerBound => {
    //                    if entry.eval <= alpha {
    //                        return entry.eval;
    //                        //return alpha;
    //                    }
    //                }
    //                NodeType::UpperBound => {
    //                    if entry.eval >= beta {
    //                        return entry.eval;
    //                        //return beta;
    //                    }
    //                }
    //            }
    //        }
    //    }

    //    // Depth limit reached
    //    if depth == 0 {
    //        let eval = self.quiescence_search(alpha, beta, root_distance);
    //        //let entry = TranspositionEntry::new(depth, eval, None, NodeType::Exact, get_zobrist_hash(self));
    //        //self.transposition_table.store(entry);
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
    //        //let entry = TranspositionEntry::new(depth, eval, None, NodeType::Exact, get_zobrist_hash(self));
    //        //self.transposition_table.store(get_zobrist_hash(self), depth, eval, NodeType::Exact);
    //        return eval;
    //    }

    //    let mut best_move: Option<Move> = None;

    //    for mov in moves {
    //        self.make_move(mov);
    //        let eval = -self.negamax(depth - 1, -beta, -alpha, root_distance + 1);
    //        self.unmake_move(mov);

    //        if eval >= beta {
    //            let entry = TranspositionEntry::new(depth, beta, Some(mov), NodeType::UpperBound, get_zobrist_hash(self));
    //            self.transposition_table.store(entry);
    //            return beta;
    //        }
    //        if eval > alpha {
    //            alpha = eval;
    //            if root_distance == 1 {
    //                best_move = Some(mov);
    //            }
    //        }
    //    }
    //    let entry = TranspositionEntry::new(depth, alpha, best_move, NodeType::LowerBound, get_zobrist_hash(self));
    //    self.transposition_table.store(entry);
    //    alpha
    //}

    // Function to retrieve the principal variation from the transposition table
    fn principal_variation(&mut self, depth: u32) -> Vec<Move> {
        let mut pv_moves = Vec::new();
        let mut hash = get_zobrist_hash(self);

        println!("{depth}");
        for curr_depth in 1..=depth {
            if let Some(entry) = self.transposition_table.probe(hash) {
                println!("{} {}", entry.depth, curr_depth);
                if entry.depth >= curr_depth && entry.hash == hash {
                    println!("{}", entry.best_move);
                    pv_moves.push(entry.best_move);
                    self.make_move(entry.best_move);
                    hash = get_zobrist_hash(self);
                    continue;
                }
            }
            break;
        }
        for mov in pv_moves.clone() {
            self.unmake_move(mov);
        }
        pv_moves
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
}
