use crate::board::bitboard::Bitboard;
use crate::board::piece::{Piece, PieceType};
use crate::board::piece_move::{Move, MoveType};
use crate::board::utils::flip_rank;
use crate::board::{Board, Side};
use crate::evaluation::evaluate;
use crate::move_generation::generate_moves;
use crate::search::book_moves::get_book_move;
use crate::search::transposition_table::{NodeType, TranspositionEntry};
use core::simd;
use std::char::MAX;
use std::cmp::Ordering;
use std::i32::{self};
use std::simd::num::SimdUint;
use std::simd::u64x8;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

pub const MAX_DEPTH: usize = 100;
pub const KILLER_MOVE_SLOTS: usize = 3;

const HASH_MOVE_SCORE: i32 = 4000;
const CAPTURE_BASE_SCORE: i32 = 1000;
const KILLER_MOVE_SCORE: i32 = 800;

const MAX_EVAL: i32 = 1000000;

#[derive(PartialEq, Copy, Clone, Default)]
pub struct SearchParams {
    pub depth: Option<u32>, // Maximum depth to search to
    //pub nodes: usize,            // Maximum number of nodes to search
    pub move_time: u128,         // Maximum time per move to search
    pub clock: Clock,            // Time available for entire game
    pub search_mode: SearchMode, // Defines the mode to search in
    pub use_book: bool,
    //pub quiet: bool,             // No intermediate search stats updates
}

pub struct Search {
    pub params: SearchParams,
    pub result: SearchResult,
    pub max_time: u128,
    pub killer_moves: [[Option<Move>; KILLER_MOVE_SLOTS]; MAX_DEPTH],
    pub pv_table: [[Option<Move>; MAX_DEPTH]; MAX_DEPTH], // Initialize PV table
    pub should_quit: Arc<AtomicBool>,             // Shared atomic flag
    pub root_ply: u32,
    pub has_searched_one_move: bool,
    pub syzygy: pyrrhic_rs::TableBases<Board>,
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
            pv_table: [[None; MAX_DEPTH]; MAX_DEPTH],
            should_quit: Arc::new(AtomicBool::new(false)),
            syzygy: pyrrhic_rs::TableBases::<Board>::new("./syzygy/tb345").unwrap(),
            has_searched_one_move: false,
        }
    }
}

impl Search {
    pub fn search(&mut self, search_params: SearchParams, board: &mut Board) -> SearchResult {
        self.should_quit.store(false, std::sync::atomic::Ordering::SeqCst);
        self.result = SearchResult::default();

        if search_params.use_book {
            if let Some(book_move) = get_book_move(board, 1.0) {
                self.result.pv.push(book_move);
                return self.result.clone();
            }
        }

        self.params = search_params;
        self.killer_moves = [[None; KILLER_MOVE_SLOTS]; MAX_DEPTH];
        self.start_time = Instant::now();
        self.root_ply = board.ply;

        if board.occupied_squares.count_ones() <= 5 {
            let mut bitboards = u64x8::from_array([
                *board.side_squares[Side::White],
                *board.side_squares[Side::Black],
                *(board.piece_squares[Piece::WhiteKing] | board.piece_squares[Piece::BlackKing]),
                *(board.piece_squares[Piece::WhiteQueen] | board.piece_squares[Piece::BlackQueen]),
                *(board.piece_squares[Piece::WhiteRook] | board.piece_squares[Piece::BlackRook]),
                *(board.piece_squares[Piece::WhiteBishop] | board.piece_squares[Piece::BlackBishop]),
                *(board.piece_squares[Piece::WhiteKnight] | board.piece_squares[Piece::BlackKnight]),
                *(board.piece_squares[Piece::WhitePawn] | board.piece_squares[Piece::BlackPawn]),
            ]);
            bitboards = bitboards.swap_bytes();
            //for bitboard in bitboards.to_array().iter() {
            //    println!("{}", Bitboard(*bitboard));
            //}
            //println!("ep {}", flip_rank(board.state().en_passant_square.unwrap_or(56)) as u32);
            //println!("ep {}", board.state().halfmove_clock as u32 / 2);
            let result = self
                .syzygy
                .probe_root(
                    bitboards[0],
                    bitboards[1],
                    bitboards[2],
                    bitboards[3],
                    bitboards[4],
                    bitboards[5],
                    bitboards[6],
                    bitboards[7],
                    board.state().halfmove_clock as u32 / 2,
                    flip_rank(board.state().en_passant_square.unwrap_or(56)) as u32,
                    board.side.value() == 0,
                )
                .expect("Syzygy tablebase probe failed");
            match result.root {
                pyrrhic_rs::DtzProbeValue::Stalemate => return self.result.clone(),
                pyrrhic_rs::DtzProbeValue::Checkmate => return self.result.clone(),
                pyrrhic_rs::DtzProbeValue::Failed => eprintln!("Dtz probe failed at root"),
                pyrrhic_rs::DtzProbeValue::DtzResult(dtz_result) => {
                    let move_type = match dtz_result.promotion {
                        pyrrhic_rs::Piece::Knight => MoveType::KnightPromotion,
                        pyrrhic_rs::Piece::Bishop => MoveType::BishopPromotion,
                        pyrrhic_rs::Piece::Rook => MoveType::RookPromotion,
                        pyrrhic_rs::Piece::Queen => MoveType::QueenPromotion,
                        _ => {
                            if dtz_result.ep {
                                MoveType::EnPassant
                            } else {
                                MoveType::Normal
                            }
                        }
                    };
                    let mov = Move::new(flip_rank(dtz_result.from_square as usize), flip_rank(dtz_result.to_square as usize), move_type);
                    self.result.pv.push(mov);
                    return self.result.clone();
                }
            }
        }

        self.max_time = match search_params.search_mode {
            SearchMode::Infinite => u128::max_value(),
            SearchMode::MoveTime => search_params.move_time,
            SearchMode::Clock => self.calculate_time(board),
        };

        for depth in 1..=MAX_DEPTH as u32 {
            if self.should_quit(depth) {
                break;
            }

            self.has_searched_one_move = false;

            let eval = self.pvs::<true>(board, depth, -MAX_EVAL, MAX_EVAL, 0);

            // Don't update results if search was interrupted
            if self.should_quit(depth) {
                break;
            }

            let new_pv = self.extract_pv(depth);

            // Aspiration windows for deeper searches
            if depth >= 4 {
                let window = 50;
                let mut alpha = eval - window;
                let mut beta = eval + window;

                loop {
                    let score = self.pvs::<true>(board, depth, alpha, beta, 0);

                    if score <= alpha {
                        alpha = -MAX_EVAL;
                    } else if score >= beta {
                        beta = MAX_EVAL;
                    } else {
                        break;
                    }
                }
            }

            // Store results
            self.result.highest_eval = eval;
            self.result.depth_reached = depth;
            self.result.pv = new_pv;
            self.result.time = self.start_time.elapsed().as_millis();

            Search::print_info(&self.result);
        }

        self.result.clone()
    }

    fn pvs<const ALLOW_NULL_MOVE: bool>(&mut self, board: &mut Board, depth: u32, mut alpha: i32, mut beta: i32, ply: u32) -> i32 {
        if self.should_quit(depth) {
            return 0;
        }

        // Check for draws first
        if self.is_draw(board) {
            return 0;
        }

        self.result.nodes += 1;

        // Leaf node reached, run quiescence search
        if depth == 0 {
            //return evaluate(board);
            return self.quiescence_search(board, alpha, beta, ply);
        }

        // Transposition table lookup
        if let Some(entry) = board.transposition_table.probe(board.zobrist_hash) {
            if entry.hash == board.zobrist_hash && entry.depth >= depth {
                self.result.transpositions += 1;

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

        if board.occupied_squares.count_ones() <= 5 {
            let mut bitboards = u64x8::from_array([
                *board.side_squares[Side::White],
                *board.side_squares[Side::Black],
                *(board.piece_squares[Piece::WhiteKing] | board.piece_squares[Piece::BlackKing]),
                *(board.piece_squares[Piece::WhiteQueen] | board.piece_squares[Piece::BlackQueen]),
                *(board.piece_squares[Piece::WhiteRook] | board.piece_squares[Piece::BlackRook]),
                *(board.piece_squares[Piece::WhiteBishop] | board.piece_squares[Piece::BlackBishop]),
                *(board.piece_squares[Piece::WhiteKnight] | board.piece_squares[Piece::BlackKnight]),
                *(board.piece_squares[Piece::WhitePawn] | board.piece_squares[Piece::BlackPawn]),
            ]);
            bitboards = bitboards.swap_bytes();
            //for bitboard in bitboards.to_array().iter() {
            //    println!("{}", Bitboard(*bitboard));
            //}
            //println!("ep {}", flip_rank(board.state().en_passant_square.unwrap_or(56)) as u32);
            //println!("halfmove {}", board.state().halfmove_clock as u32 / 2);
            //println!("fen {}", board.fen());
            let result = self
                .syzygy
                .probe_root(
                    bitboards[0],
                    bitboards[1],
                    bitboards[2],
                    bitboards[3],
                    bitboards[4],
                    bitboards[5],
                    bitboards[6],
                    bitboards[7],
                    board.state().halfmove_clock as u32 / 2,
                    flip_rank(board.state().en_passant_square.unwrap_or(56)) as u32,
                    board.side.value() == 0,
                )
                .expect("Syzygy tablebase probe failed");
            match result.root {
                pyrrhic_rs::DtzProbeValue::Stalemate => return 0,
                pyrrhic_rs::DtzProbeValue::Checkmate => {
                    let king_square = board.piece_squares[Piece::new(PieceType::King, board.side) as usize].lsb();
                    if board.attacked(king_square) {
                        return -MAX_EVAL + ply as i32;
                    } else {
                        return MAX_EVAL - ply as i32;
                    };
                }
                pyrrhic_rs::DtzProbeValue::Failed => eprintln!("Dtz probe failed at root"),
                pyrrhic_rs::DtzProbeValue::DtzResult(dtz_result) => {
                    return match dtz_result.wdl {
                        pyrrhic_rs::WdlProbeResult::Loss => -MAX_EVAL,
                        pyrrhic_rs::WdlProbeResult::BlessedLoss => -MAX_EVAL + 10000,
                        pyrrhic_rs::WdlProbeResult::Draw => 0,
                        pyrrhic_rs::WdlProbeResult::CursedWin => MAX_EVAL - 10000,
                        pyrrhic_rs::WdlProbeResult::Win => MAX_EVAL,
                    };
                }
            }
        }

        let reduction = 2;
        if ALLOW_NULL_MOVE && !board.in_check() && depth > reduction + 1 {
            board.make_null_move();
            let null_move_eval = -self.pvs::<false>(board, depth - 1 - reduction, -beta, -beta + 1, ply + 1);
            board.unmake_null_move();

            if null_move_eval >= beta {
                return null_move_eval; // Beta cutoff, prune this node
            }
        }

        // Mate distance pruning
        alpha = alpha.max(-MAX_EVAL + ply as i32);
        beta = beta.min(MAX_EVAL - ply as i32);
        if alpha >= beta {
            return alpha;
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

        let mut best_move = None;
        let mut best_eval = -MAX_EVAL;
        let mut evaluation_bound = NodeType::UpperBound;

        // Principal Variation Search
        for (i, &mov) in moves.iter().enumerate() {
            board.make_move(mov);

            let eval = if i == 0 {
                -self.pvs::<true>(board, depth - 1, -beta, -alpha, ply + 1)
            } else {
                let mut eval = -self.pvs::<true>(board, depth - 1, -(alpha + 1), -alpha, ply + 1);
                if eval > alpha && eval < beta {
                    eval = -self.pvs::<true>(board, depth - 1, -beta, -alpha, ply + 1);
                }
                eval
            };

            board.unmake_move(mov);

            if ply == 0 {
                self.has_searched_one_move = true;
            }

            if eval > best_eval {
                best_eval = eval;
                best_move = Some(mov);

                if eval > alpha {
                    evaluation_bound = NodeType::Exact;
                    alpha = eval;
                    self.pv_table[ply as usize][ply as usize] = best_move;
                    for i in (ply + 1)..MAX_DEPTH as u32 {
                        self.pv_table[ply as usize][i as usize] = self.pv_table[(ply + 1) as usize][i as usize];
                    }
                }
            }

            if eval >= beta {
                evaluation_bound = NodeType::LowerBound;
                if !board.is_capture(mov) {
                    self.update_killer_moves(mov, ply);
                }
                break;
            }
        }

        // Store position in transposition table
        if let Some(best_move) = best_move {
            let entry = TranspositionEntry::new(depth, best_eval, best_move, evaluation_bound, board.zobrist_hash);
            board.transposition_table.store(entry);
        }

        best_eval
    }

    fn quiescence_search(&mut self, board: &mut Board, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        if self.should_quit(ply) {
            return 0;
        }

        // Stand pat evaluation
        let stand_pat = evaluate(board);
        if stand_pat >= beta {
            return beta;
        }

        alpha = alpha.max(stand_pat);

        // Generate and filter captures
        let mut moves = generate_moves(board);
        moves.retain(|mov| board.is_capture(*mov));

        if moves.is_empty() {
            return stand_pat;
        }

        self.result.nodes += 1;

        self.order_moves::<true>(board, &mut moves, ply);

        for mov in moves {
            board.make_move(mov);
            let eval = -self.quiescence_search(board, -beta, -alpha, ply + 1);
            board.unmake_move(mov);

            if eval >= beta {
                return beta;
            }
            alpha = alpha.max(eval);
        }

        alpha
    }

    fn is_draw(&self, board: &Board) -> bool {
        if board.state().halfmove_clock >= 100 {
            return true;
        }

        if board.can_detect_threefold_repetition && board.ply - board.state().last_irreversible_ply >= 4 {
            let mut count = 0;
            let mut current_ply = (board.ply - 2) as i32;

            while current_ply >= board.state().last_irreversible_ply as i32 {
                if board.states[current_ply as usize].zobrist_hash == board.zobrist_hash {
                    count += 1;
                    if count >= 2 || (count == 1 && board.ply > self.root_ply + 2) {
                        return true;
                    }
                }
                current_ply -= 2;
            }
        }

        false
    }

    pub fn extract_pv(&self, depth: u32) -> Vec<Move> {
        let mut pv = Vec::new();
        for i in 0..depth as usize {
            if let Some(mov) = self.pv_table[0][i] {
                pv.push(mov);
            } else {
                break;
            }
        }
        pv
    }

    pub fn order_moves<const ONLY_CAPTURES: bool>(&self, board: &Board, moves: &mut [Move], ply: u32) {
        moves.sort_by_cached_key(|mov| -self.get_move_score::<ONLY_CAPTURES>(*mov, board, ply));
    }

    // TODO: Just realized this is incredibly inefficient, get_move_score is slow already and is
    // being called multiple times on the same move in the same sort.
    fn get_move_score<const ONLY_CAPTURES: bool>(&self, mov: Move, board: &Board, ply: u32) -> i32 {
        if let Some(pv_mov) = self.pv_table[ply as usize][ply as usize] {
            if mov == pv_mov {
                return MAX_EVAL;
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
