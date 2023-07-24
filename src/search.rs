use crate::{
    board::{Board, Side},
    piece::{Piece, PieceType},
    piece_move::Move,
};

pub struct SearchOption {
    pub depth: u32,
    pub infinite: bool,
}

pub struct SearchResult {
    pub best_move: Move,
    pub eval: i32,
}

impl Board {
    pub fn search(&mut self, search_option: SearchOption) -> Move {
        let mut highest_eval = i32::MIN;
        let mut best_move: Option<Move> = None;

        let moves = self.generate_moves();
        for mov in moves {
            self.make_move(mov);
            let eval = -self.alpha_beta(i32::MIN + 1, i32::MAX, search_option.depth - 1, 1);
            self.unmake_move(mov);
            println!("{mov}: {eval}");
            if eval > highest_eval {
                highest_eval = eval;
                best_move = Some(mov);
            }
        }
        best_move.unwrap()
    }

    pub fn alpha_beta(&mut self, mut alpha: i32, beta: i32, depth: u32, root_distance: u32) -> i32 {
        if depth == 0 {
            return self.quiescense_search(alpha, beta, root_distance);
            //return self.evaluate();
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

        for mov in moves {
            self.make_move(mov);
            let score = -self.alpha_beta(-beta, -alpha, depth - 1, root_distance + 1);
            self.unmake_move(mov);

            if score >= beta {
                return beta; // Beta cutoff, as this move results in a position too good for the opponent
            }
            if score > alpha {
                alpha = score; // Update alpha with the new score
            }
        }
        alpha
    }
    pub fn quiescense_search(&mut self, mut alpha: i32, beta: i32, root_distance: u32) -> i32 {
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
                let eval = -self.quiescense_search(-beta, -alpha, root_distance + 1);
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
