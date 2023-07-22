use crate::{board::Board, piece_move::Move};

pub struct SearchOption {
    pub depth: u32,
    pub infinite: bool,
}

impl Board {
    pub fn search(&mut self, search_option: SearchOption) -> Move {
        let mut highest_score = i32::MIN;
        let mut best_move: Option<Move> = None;
        let moves = self.generate_moves();
        for mov in moves {
            self.make_move(mov);
            let score = self.alpha_beta_search(i32::MIN, i32::MAX, search_option.depth - 1);
            println!("{mov} : {score}");
            self.unmake_move(mov);

            if score > highest_score {
                highest_score = score;
                best_move = Some(mov);
            }
        }
        best_move.unwrap()
    }
    pub fn alpha_beta_search(&mut self, mut alpha: i32, beta: i32, depth_left: u32) -> i32 {
        if depth_left == 0 {
            //return self.quiescense_search(alpha, beta);
            return self.evaluate();
        }
        for mov in self.generate_moves() {
            self.make_move(mov);
            let score = -self.alpha_beta_search(-beta, -alpha, depth_left - 1);
            self.unmake_move(mov);
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
            if Option::is_some(&self.state().captured_piece) {
                self.make_move(mov);
                let score = -self.quiescense_search(-beta, -alpha);
                self.unmake_move(mov);

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
