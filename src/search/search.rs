use crate::board::piece::{Piece, PieceType};
use crate::board::piece_move::Move;
use crate::board::Board;
use crate::evaluation::evaluate;
use crate::move_generation::generate_moves;
use crate::search::book_moves::get_book_move;
use crate::search::transposition_table::{NodeType, TranspositionEntry};
use std::cmp::Ordering;
use std::i32;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

pub const MAX_DEPTH: usize = 100;
pub const KILLER_MOVE_SLOTS: usize = 3;

const HASH_MOVE_SCORE: i32 = 1200;
const CAPTURE_BASE_SCORE: i32 = 1000;
const KILLER_MOVE_SCORE: i32 = 2000;

const MAX_EVAL: i32 = 100000000;

#[derive(PartialEq, Copy, Clone, Default)]
pub struct SearchParams {
    pub depth: Option<u32>, // Maximum depth to search to
    //pub nodes: usize,            // Maximum number of nodes to search
    pub move_time: u128, // Maximum time per move to search
    pub clock: Clock,    // Time available for entire game
    pub search_mode: SearchMode, // Defines the mode to search in
                         //pub quiet: bool,             // No intermediate search stats updates
}

pub struct Search {
    pub params: SearchParams,
    pub result: SearchResult,
    pub max_time: u128,
    pub killer_moves: [[Option<Move>; KILLER_MOVE_SLOTS]; MAX_DEPTH],
    pub pv: Vec<Move>,
    pub should_quit: Arc<AtomicBool>, // Shared atomic flag
    pub root_ply: u32,
    pub has_searched_one_move: bool,
    start_time: Instant,
}

impl Search {
    pub fn should_quit(&self, ply: u32) -> bool {
        if self.start_time.elapsed().as_millis() > self.max_time {
            return true;
        }
        if let Some(max_depth) = self.params.depth {
            if ply > max_depth {
                return true;
            }
        }
        return self.should_quit.load(std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(PartialEq, Copy, Clone, Default)]
pub struct Clock {
    pub time: [u128; 2],            // Time on the clock in milliseconds
    pub inc: [u128; 2],             // Time increment in milliseconds
    pub moves_to_go: Option<usize>, // Moves to go to next time control (0 = sudden death)
}

#[derive(PartialEq, Copy, Clone)]
pub enum SearchMode {
    Infinite,
    MoveTime,
    Clock,
}

impl Default for SearchMode {
    fn default() -> Self {
        Self::Infinite
    }
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

impl Default for Search {
    fn default() -> Self {
        Self {
            params: SearchParams::default(),
            result: SearchResult::default(),
            root_ply: 0,
            start_time: Instant::now(),
            max_time: 0,
            killer_moves: [[None; KILLER_MOVE_SLOTS]; MAX_DEPTH],
            pv: Vec::with_capacity(MAX_DEPTH),
            should_quit: Arc::new(AtomicBool::new(false)),
            has_searched_one_move: false,
        }
    }
}

impl Search {
    pub fn search(&mut self, search_params: SearchParams, board: &mut Board) -> SearchResult {
        self.should_quit.store(false, std::sync::atomic::Ordering::SeqCst);

        //if let Some(book_move) = get_book_move(board, 1.0) {
        //    self.result.pv.push(book_move);
        //    return self.result.clone();
        //}

        self.params = search_params;
        self.pv.clear();
        self.killer_moves = [[None; KILLER_MOVE_SLOTS]; MAX_DEPTH];
        self.start_time = Instant::now();
        self.root_ply = board.ply;

        // TODO: Add infinite mode as a const parameter to search instead of using u32::max_value
        self.max_time = match search_params.search_mode {
            SearchMode::Infinite => u128::max_value(),
            SearchMode::MoveTime => search_params.move_time,
            SearchMode::Clock => self.calculate_time(board),
        };
        println!("search time: {} ms", self.max_time);

        //let vote_map = [0; 64 * 64];
        //let available_threads: usize = thread::available_parallelism().unwrap().into();
        //let mut threads = Vec::with_capacity(available_threads);

        //for i in 0..available_threads {
        //    threads.push(thread::spawn(move || {}));
        //}

        //board.transposition_table.clear();


        //let mut old_result = SearchResult::default();
        for depth in 1..=MAX_DEPTH as u32 {
            if self.should_quit(depth) {
                break;
            }

            self.has_searched_one_move = false;

            //old_result = self.result.clone();

            let eval = self.pvs(board, depth, -MAX_EVAL, MAX_EVAL, 0);

            self.result.highest_eval = eval;
            self.result.depth_reached = depth;
            self.result.pv = self.extract_pv(depth, board);
            self.result.time = self.start_time.elapsed().as_millis();
            Search::print_info(&self.result);
        }
        self.result.clone()
    }

    // Principal variation search using fail-soft alpha beta search
    fn pvs(&mut self, board: &mut Board, depth: u32, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        if self.should_quit(depth) {
            return 0;
        }

        self.result.nodes += 1;

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

        if board.state().halfmove_clock >= 100 {
            return 0;
        }

        // TODO: Make this const generic
        if board.can_detect_threefold_repetition {
            if board.ply - board.state().last_irreversible_ply >= 4 {
                let mut ply = (board.ply - 2) as i32;
                let mut count = 0;
                while ply >= board.state().last_irreversible_ply as i32 {
                    let state = &board.states[board.ply as usize];
                    if state.zobrist_hash == board.zobrist_hash {
                        count += 1;
                    }
                    ply -= 2;
                }
                let is_draw = (count == 2) || (count == 1 && board.ply > self.root_ply + 2);
                if is_draw {
                    return 0;
                }
            }
        }

        // Leaf node reached, so we run quiescence search
        if depth == 0 {
            return self.quiescence_search(board, alpha, beta, ply);
        }

        let mut moves = generate_moves(board);

        // Check for terminal positions
        if moves.is_empty() {
            let king_square = board.piece_squares[Piece::new(PieceType::King, board.side) as usize].lsb();
            return if board.attacked(king_square) {
                -MAX_EVAL + ply as i32
            } else {
                0 // Stalemate
            };
        }

        self.order_moves::<false>(board, &mut moves, ply);

        let mut best_move: Option<Move> = None;
        let mut evaluation_bound = NodeType::UpperBound;
        let mut best_eval = i32::MIN;

        // Search remaining moves with zero window
        for (i, mov) in moves.iter().enumerate() {
            board.make_move(*mov);

            let eval = if i == 0 {
                // The first move is searched normally
                -self.pvs(board, depth - 1, -beta, -alpha, ply + 1)
            } else {
                // Try null-window search first
                let mut eval = -self.pvs(board, depth - 1, -alpha - 1, -alpha, ply + 1);

                // If the null-window search failed high, do a full re-search
                if eval > alpha && eval < beta {
                    eval = -self.pvs(board, depth - 1, -beta, -alpha, ply + 1);
                }

                eval
            };

            board.unmake_move(*mov);

            if ply == 0 {
                self.has_searched_one_move = true;
            }

            if eval > best_eval {
                best_eval = eval;
                best_move = Some(*mov);

                if eval > alpha {
                    // We found a move better than alpha and update alpha
                    evaluation_bound = NodeType::Exact;
                    alpha = eval;
                }
            }

            if eval >= beta {
                // Fail soft beta-cutoff: move is too good so the opponent refutes this line, we
                // therefore exit
                evaluation_bound = NodeType::LowerBound;

                // Killer Move is a quiet move which caused a beta-cutoff in a sibling Cut-node(LowerBound-node), or any other earlier branch in the tree with the same ply distance to the root
                if !board.is_capture(*mov) {
                    self.update_killer_moves(*mov, ply);
                }
                break;
            }
        }

        let entry = TranspositionEntry::new(depth, best_eval, best_move.unwrap(), evaluation_bound, board.zobrist_hash);
        board.transposition_table.store(entry);

        best_eval
    }

    fn quiescence_search(&mut self, board: &mut Board, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        if self.should_quit(ply) {
            return 0;
        }

        self.result.nodes += 1;

        if board.state().halfmove_clock >= 100 {
            return 0;
        }

        // TODO: Make this const generic
        if board.can_detect_threefold_repetition {
            if board.ply - board.state().last_irreversible_ply >= 4 {
                let mut ply = (board.ply - 2) as i32;
                let mut count = 0;
                while ply >= board.state().last_irreversible_ply as i32 {
                    let state = &board.states[ply as usize];
                    if state.zobrist_hash == board.zobrist_hash {
                        count += 1;
                    }
                    ply -= 2;
                }
                let is_draw = (count == 2) || (count == 1 && board.ply > self.root_ply + 2);
                if is_draw {
                    return 0;
                }
            }
        }

        let stand_pat = evaluate(board);
        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        let mut moves = generate_moves(board);
        if moves.is_empty() {
            let king_square = board.piece_squares[Piece::new(PieceType::King, board.side) as usize].lsb();
            return if board.attacked(king_square) {
                -MAX_EVAL + ply as i32
            } else {
                0 // Stalemate
            };
        }
        moves.retain(|mov| board.is_capture(*mov));
        self.order_moves::<true>(board, &mut moves, ply);

        let mut best_eval = stand_pat;

        for mov in moves {
            board.make_move(mov);
            let eval = -self.quiescence_search(board, -beta, -alpha, ply + 1);
            board.unmake_move(mov);

            if eval > best_eval {
                best_eval = eval;

                if eval > alpha {
                    // We found a move better than alpha and update alpha
                    alpha = eval;
                }
            }

            if eval >= beta {
                // Fail soft beta-cutoff: move is too good so the opponent refutes this line, we
                // therefore exit
                break;
            }
        }

        return best_eval;
    }

    pub fn extract_pv(&mut self, depth: u32, board: &mut Board) -> Vec<Move> {
        let mut current_hash = board.zobrist_hash;
        let mut pv = Vec::new();
        let mut i = 0;

        while let Some(entry) = board.transposition_table.probe(current_hash) {
            if entry.hash != current_hash || i >= depth {
                break;
            }

            let pv_move = entry.best_move;
            pv.push(pv_move);

            // Make the move on the board to get the next position
            board.make_move(pv_move);
            current_hash = board.zobrist_hash;
            i += 1;
        }

        // Unmake all the moves to restore the original board state
        for &mov in pv.iter().rev() {
            board.unmake_move(mov);
        }
        pv
    }

    pub fn order_moves<const ONLY_CAPTURES: bool>(&self, board: &Board, moves: &mut [Move], ply: u32) {
        moves.sort_by_cached_key(|mov| -self.get_move_score::<ONLY_CAPTURES>(*mov, board, ply));
    }

    // TODO: Just realized this is incredibly inefficient, get_move_score is slow already and is
    // being called multiple times on the same move in the same sort.
    fn get_move_score<const ONLY_CAPTURES: bool>(&self, mov: Move, board: &Board, ply: u32) -> i32 {
        if let Some(pv_mov) = self.pv.get(ply as usize) {
            if mov == *pv_mov {
                return i32::MAX;
            }
        }

        let mut score = 0;

        if ONLY_CAPTURES || board.is_capture(mov) {
            let captured_piece = board.squares[mov.to()].unwrap();
            let moving_piece = board.squares[mov.from()].unwrap();
            let capture_score = captured_piece.piece_type().centipawns() - moving_piece.piece_type().centipawns();
            score += CAPTURE_BASE_SCORE + capture_score;
        } else {
            for killer_move in &self.killer_moves[ply as usize] {
                if let Some(killer) = killer_move {
                    if mov == *killer {
                        score += KILLER_MOVE_SCORE;
                        break;
                    }
                }
            }
        }
        if let Some(entry) = board.transposition_table.probe(board.zobrist_hash) {
            if entry.best_move == mov {
                score += HASH_MOVE_SCORE;
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

    fn calculate_time(&mut self, board: &Board) -> u128 {
        let half_moves_left = Search::remaining_half_moves(board.total_material) as u128;
        let time_left = self.params.clock.time[board.side] + self.params.clock.inc[board.side] * half_moves_left / 2;
        (time_left / half_moves_left) / 2
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

    pub fn print_info(result: &SearchResult) {
        print!(
            "info depth {} score cp {} time {} nodes {} nps {} pv ",
            result.depth_reached,
            result.highest_eval,
            result.time,
            result.nodes,
            (result.nodes as f32 / result.time as f32 * 1000.0) as u32
        );
        for mov in result.pv.iter() {
            print!("{} ", mov);
        }
        println!();
    }
}
