use crate::{
    board::{Board, Side},
    piece::{Piece, PieceType},
    piece_move::Move,
};

pub struct SearchOption {
    pub depth: u32,
    pub infinite: bool,
}

impl Board {
    pub fn search(&mut self, search_option: SearchOption) -> Move {
        let mut highest_eval = i32::MIN;
        let mut best_move: Option<Move> = None;
        let mut maximizing_player = true;

        let moves = self.generate_moves();
        for mov in moves {
            self.make_move(mov);
            let eval = -self.minimax(search_option.depth - 1, i32::MIN, i32::MAX, true);
            self.unmake_move(mov);
            println!("{mov}: {eval}");
            if eval > highest_eval {
                highest_eval = eval;
                best_move = Some(mov);
            }
        }
        best_move.unwrap()
    }
    pub fn minimax(&mut self, depth: u32, mut alpha: i32, mut beta: i32, maximizing_player: bool) -> i32 {
        if depth == 0 {
            return self.evaluate();
        }
        let moves = self.generate_moves();
        if moves.len() == 0 {
            let king_square = self.piece_squares[Piece::new(PieceType::King, self.side_to_move)].lsb();
            if self.attacked(king_square) {
                return if maximizing_player { i32::MIN } else { i32::MAX };
            } else {
                return 0;
            }
        }
        if maximizing_player {
            let mut max_eval = i32::MIN;
            for mov in moves {
                self.make_move(mov);
                let eval = self.minimax(depth - 1, alpha, beta, false);
                self.unmake_move(mov);
                max_eval = i32::max(max_eval, eval);
                alpha = i32::max(alpha, eval);
                if beta <= alpha {
                    break;
                }
            }
            return max_eval;
        } else {
            let mut min_eval = i32::MAX;
            for mov in moves {
                self.make_move(mov);
                let eval = self.minimax(depth - 1, alpha, beta, true);
                self.unmake_move(mov);
                min_eval = i32::min(min_eval, eval);
                beta = i32::min(beta, eval);
                if beta <= alpha {
                    break;
                }
            }
            return min_eval;
        }
    }
    //pub fn search(&mut self, search_option: SearchOption) -> Move {
    //    let mut highest_eval = i32::MIN;
    //    let mut best_move: Option<Move> = None;
    //    let mut maximizing_player = true;

    //    let moves = self.generate_moves();
    //    for mov in moves {
    //        self.make_move(mov);
    //        let eval = -self.alpha_beta(search_option.depth - 1, i32::MIN, i32::MAX);
    //        self.unmake_move(mov);
    //        println!("{mov}: {eval}");
    //        if eval > highest_eval {
    //            highest_eval = eval;
    //            best_move = Some(mov);
    //        }
    //    }
    //    best_move.unwrap()
    //}

    ////pub fn alpha_beta(&mut self, depth: u32, mut alpha: i32, mut beta: i32) -> i32 {
    ////    if depth == 0 {
    ////        return self.evaluate();
    ////    }
    ////    let moves = self.generate_moves();
    ////    if moves.len() == 0 {
    ////        let king_square = self.piece_squares[Piece::new(PieceType::King, self.side_to_move)].lsb();
    ////        if self.attacked(king_square) {
    ////            return i32::MIN;
    ////        } else {
    ////            return 0;
    ////        }
    ////    }

    ////    let mut max_eval = i32::MIN;
    ////    for mov in moves {
    ////        self.make_move(mov);
    ////        let eval = -self.alpha_beta(depth - 1, -beta, -alpha);
    ////        self.unmake_move(mov);
    ////        max_eval = i32::max(max_eval, eval);
    ////        alpha = i32::max(alpha, eval);
    ////        if alpha >= beta {
    ////            break;
    ////        }
    ////    }
    ////    max_eval
    ////}
    //pub fn alpha_beta(&mut self, depth: u32, mut alpha: i32, mut beta: i32) -> i32 {
    //    if depth == 0 {
    //        //return self.quiescense_search(alpha, beta);
    //        return self.evaluate();
    //    }
    //    let moves = self.generate_moves();
    //    if moves.len() == 0 {
    //        let king_square = self.piece_squares[Piece::new(PieceType::King, self.side_to_move)].lsb();
    //        if self.attacked(king_square) {
    //            return i32::MIN;
    //        } else {
    //            return 0;
    //        }
    //    }

    //    for mov in moves {
    //        self.make_move(mov);
    //        let eval = -self.alpha_beta(depth - 1, -beta, -alpha);
    //        self.unmake_move(mov);

    //        alpha = i32::max(alpha, eval);
    //        if alpha >= beta {
    //            break;
    //        }
    //    }
    //    alpha
    //}
    //pub fn quiescense_search(&mut self, mut alpha: i32, beta: i32) -> i32 {
    //    let stand_pat = self.evaluate();
    //    if stand_pat >= beta {
    //        return beta;
    //    }
    //    if alpha < stand_pat {
    //        alpha = stand_pat;
    //    }
    //    let moves = self.generate_moves();
    //    if moves.len() == 0 {
    //        let king_square = self.piece_squares[Piece::new(PieceType::King, self.side_to_move)].lsb();
    //        if self.attacked(king_square) {
    //            return i32::MIN;
    //        } else {
    //            return 0;
    //        }
    //    }
    //    for mov in moves {
    //        if self.is_capture(mov) {
    //            self.make_move(mov);
    //            let eval = -self.quiescense_search(-beta, -alpha);
    //            self.unmake_move(mov);

    //            alpha = i32::max(alpha, eval);
    //            if alpha >= beta {
    //                break;
    //            }
    //        }
    //    }
    //    alpha
    //}
}
