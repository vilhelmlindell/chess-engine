use crate::{board::Board, r#move::Move};

impl Board {
    fn search(&self, alpha: i32, beta: i32, depth_left: u32) {
        if depth_left == 0 {
            return self.quiescense_search(alpha, beta, depth_left - 1);
        }
        let moves = self.generate_moves();
        for r#move in moves {
            let score = -self.alpha_beta_search(0 - alpha, -beta, depth_left - 1);
        }
    }
    fn quiescense_search(&self, alpha: i32, beta: i32, depth_left: u32) {}
}
