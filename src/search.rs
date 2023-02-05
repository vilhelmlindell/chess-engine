use crate::evaluation;
use crate::move_generation;
use crate::{board::Board, piece_move::Move};

impl Board {
    pub fn alpha_beta_search(&mut self, mut alpha: i32, beta: i32, depth_left: u32) -> i32 {
        if depth_left == 0 {
            return self.quiescense_search(alpha, beta);
        }
        for mov in self.generate_moves() {
            let score = -self.alpha_beta_search(0 - alpha, -beta, depth_left - 1);
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }
    pub fn quiescense_search(&mut self, mut alpha: i32, beta: i32) -> i32 {
        let stand_pat = self.evaluate();
        if stand_pat >= beta {
            return beta;
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }
        for mov in self.generate_moves() {
            if mov.captured_piece != None {
                self.make_move(&mov);
                let score = -self.quiescense_search(-beta, -alpha);
                self.unmake_move(&mov);

                if score >= beta {
                    return beta;
                }
                if score > alpha {
                    alpha = score;
                }
            }
        }
        alpha
    }
}
