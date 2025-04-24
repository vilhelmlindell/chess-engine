use pyrrhic_rs::DtzProbeResult;

use crate::board::bitboard::Bitboard;
use crate::board::piece::{Piece, PieceType};
use crate::board::piece_move::{Move, MoveType};
use crate::board::utils::flip_rank;
use crate::board::{Board, Side};
use crate::evaluation::evaluate;
use crate::move_generation::generate_moves;
use crate::search::book_moves::get_book_move;
use crate::search::transposition_table::{Bound, TranspositionEntry};
use core::{hash, simd};
use std::char::MAX;
use std::cmp::Ordering;
use std::i32::{self};
use std::ops::Deref;
use std::simd::num::SimdUint;
use std::simd::u64x8;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::move_ordering::order_moves;
use super::transposition_table;

pub const MAX_DEPTH: usize = 100;
pub const KILLER_MOVE_SLOTS: usize = 3;

const MAX_EVAL: i32 = 20000;

// Transposition Table
#[cfg(feature = "tt")]
pub const USE_TT: bool = true;
#[cfg(not(feature = "tt"))]
pub const USE_TT: bool = false;

// Late Move Reduction
#[cfg(feature = "lmr")]
pub const USE_LMR: bool = true;
#[cfg(not(feature = "lmr"))]
pub const USE_LMR: bool = false;

// Syzygy endgame tablebases
#[cfg(feature = "syzygy")]
pub const USE_SYZYGY: bool = true;
#[cfg(not(feature = "syzygy"))]
pub const USE_SYZYGY: bool = false;

// Quiescence search
#[cfg(feature = "quiescence")]
pub const USE_QUIESCENCE: bool = true;
#[cfg(not(feature = "quiescence"))]
pub const USE_QUIESCENCE: bool = false;

// Null Move pruning
#[cfg(feature = "null-move")]
pub const USE_NULL_MOVE: bool = true;
#[cfg(not(feature = "null-move"))]
pub const USE_NULL_MOVE: bool = false;

// Move ordering
#[cfg(feature = "move-ordering")]
pub const USE_MOVE_ORDERING: bool = true;
#[cfg(not(feature = "move-ordering"))]
pub const USE_MOVE_ORDERING: bool = false;

// Principal Variation Search
#[cfg(feature = "pvs")]
pub const USE_PVS: bool = true;
#[cfg(not(feature = "pvs"))]
pub const USE_PVS: bool = false;

// Alpha-Beta pruning
#[cfg(feature = "alpha-beta")]
pub const USE_ALPHA_BETA: bool = true;
#[cfg(not(feature = "alpha-beta"))]
pub const USE_ALPHA_BETA: bool = false;

// Aspiration windows
#[cfg(feature = "aspiration")]
pub const USE_ASPIRATION: bool = true;
#[cfg(not(feature = "aspiration"))]
pub const USE_ASPIRATION: bool = false;

// Futility pruning
// #[cfg(feature = "futility")]
// pub const USE_FUTILITY: bool = true;
// #[cfg(not(feature = "futility"))]
// pub const USE_FUTILITY: bool = false;

// Iterative deepening
//#[cfg(feature = "iterative-deepening")]
//pub const USE_ITERATIVE_DEEPENING: bool = true;
//#[cfg(not(feature = "iterative-deepening"))]
//pub const USE_ITERATIVE_DEEPENING: bool = false;
pub const USE_ITERATIVE_DEEPENING: bool = true;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum NodeType {
    Root,
    PV,
    NonPV,
}

pub struct Search {
    pub params: SearchParams,
    pub result: SearchResult,
    pub max_time: u128,
    pub killer_moves: [[Option<Move>; KILLER_MOVE_SLOTS]; MAX_DEPTH],
    pub pv_table: [[Option<Move>; MAX_DEPTH]; MAX_DEPTH], // Initialize PV table
    pub pv_lengths: [usize; MAX_DEPTH],
    //pub pv: Vec<Move>,
    pub should_quit: Arc<AtomicBool>, // Shared atomic flag
    pub root_ply: u32,
    pub syzygy: pyrrhic_rs::TableBases<Board>,
    pub start_time: Instant,
    pub previous_static_eval: i32,
    pub history: [[[u32; 64]; 64]; 2],
}

impl Search {
    pub fn search(&mut self, search_params: SearchParams, board: &mut Board) -> SearchResult {
        self.should_quit.store(false, std::sync::atomic::Ordering::SeqCst);
        self.result = SearchResult::default();
        self.pv_table = [[None; MAX_DEPTH]; MAX_DEPTH];
        self.pv_lengths = [0; MAX_DEPTH];
        //board.transposition_table.clear();

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

        if USE_SYZYGY && board.occupied_squares.count_ones() <= 5 {
            let result = self.probe_syzygy_root(board);
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
                    self.result.pv.clear();
                    self.result.pv.push(mov);
                    Search::print_info(&self.result);
                    return self.result.clone();
                }
            }
        }

        self.max_time = match search_params.search_mode {
            SearchMode::Infinite => u128::max_value(),
            SearchMode::MoveTime => search_params.move_time,
            SearchMode::Clock => self.calculate_time(board),
        };
        //self.max_time = 3000;

        // Modify search logic based on USE_ITERATIVE_DEEPENING constant
        if USE_ITERATIVE_DEEPENING {
            for depth in 1..=MAX_DEPTH as u32 {
                let mut eval;

                if !USE_ASPIRATION || depth < 4 {
                    eval = self.pvs::<{ NodeType::Root as u8 }, false>(board, depth, -MAX_EVAL, MAX_EVAL, 0);
                } else {
                    const WINDOW_SIZE: i32 = 33;

                    // Aspiration windows for deeper searches
                    let mut alpha = self.result.highest_eval - WINDOW_SIZE;
                    let mut beta = self.result.highest_eval + WINDOW_SIZE;
                    let mut window_size = WINDOW_SIZE;

                    // Iterative deepening with gradually expanding windows
                    loop {
                        eval = self.pvs::<{ NodeType::Root as u8 }, false>(board, depth, alpha, beta, 0);
                        // Handle window failures
                        if eval <= alpha {
                            // Lower bound failure - expand window downward
                            window_size = window_size * 2;
                            alpha = i32::max(eval - window_size, -MAX_EVAL);
                        } else if eval >= beta {
                            // Upper bound failure - expand window upward
                            window_size = window_size * 2;
                            beta = i32::min(eval + window_size, MAX_EVAL);
                        } else {
                            // Search successful within window
                            break;
                        }

                        // Safety check for too many window re-searches
                        if window_size >= MAX_EVAL / 2 {
                            // If windows are getting too large, do a full-width search
                            eval = self.pvs::<{ NodeType::Root as u8 }, false>(board, depth, -MAX_EVAL, MAX_EVAL, 0);
                            break;
                        }
                    }
                }

                if self.should_quit(depth) {
                    break;
                }

                self.result.highest_eval = eval;
                self.result.depth_reached = depth;
                self.result.pv = self.extract_pv();
                self.result.time = self.start_time.elapsed();
                Search::print_info(&self.result);
            }
        } else {
            unimplemented!("iterative deepening needs to be enabled");
            // Non-iterative deepening: search to the maximum depth directly
            let eval = self.pvs::<{ NodeType::Root as u8 }, false>(board, MAX_DEPTH as u32, -MAX_EVAL, MAX_EVAL, 0);

            self.result.highest_eval = eval;
            self.result.depth_reached = MAX_DEPTH as u32;
            self.result.pv = self.extract_pv();
            self.result.time = self.start_time.elapsed();
            Search::print_info(&self.result);
        }

        self.result.clone()
    }
    fn pvs<const NODE_TYPE: u8, const IS_NULL: bool>(&mut self, board: &mut Board, depth: u32, mut alpha: i32, mut beta: i32, ply: u32) -> i32 {
        let on_pv = NODE_TYPE != NodeType::NonPV as u8;
        let is_root = NODE_TYPE == NodeType::Root as u8;

        self.pv_lengths[ply as usize] = 0;

        if self.should_quit(depth) {
            return 0;
        }

        if board.state().halfmove_clock >= 100 {
            return 0;
        }

        if !is_root && board.is_repetition(on_pv) {
            return 0;
        }

        if depth == 0 {
            if USE_QUIESCENCE {
                return self.quiescence_search(board, alpha, beta, ply);
            } else {
                return evaluate(board);
            }
        }

        alpha = alpha.max(-MAX_EVAL + ply as i32);
        beta = beta.min(MAX_EVAL - ply as i32);
        if alpha >= beta {
            return alpha;
        }

        self.result.nodes += 1;

        let mut hash_move = None;

        if USE_TT {
            if let Some(entry) = transposition_table::global_probe(board.zobrist_hash) {
                //if entry.hash == board.zobrist_hash && entry.depth as u32 >= depth && !is_root && NODE_TYPE == NodeType::NonPV as u8 && !IS_NULL {
                if entry.hash == board.zobrist_hash && entry.depth as u32 >= depth && !is_root && !IS_NULL {
                    hash_move = Some(entry.best_move);
                    self.result.transpositions += 1;

                    //match entry.node_type {
                    //    Bound::Exact => {
                    //        self.result.transpositions_exact += 1;
                    //        return entry.eval as i32;
                    //    },
                    //    Bound::Lower => {
                    //        self.result.transpositions_lower += 1;
                    //        if entry.eval as i32 >= beta { return entry.eval as i32 }},
                    //    Bound::Upper => {
                    //        self.result.transpositions_upper += 1;
                    //        if entry.eval as i32 <= alpha { return entry.eval  as i32 }},
                    //}

                    match entry.node_type {
                        Bound::Exact => return entry.eval as i32,
                        Bound::Lower => alpha = alpha.max(entry.eval as i32),
                        Bound::Upper => beta = beta.min(entry.eval as i32),
                    }

                    if alpha >= beta {
                        return entry.eval as i32;
                    }
                }
            }
        }

        //let hash_move = tt_hit.map(|entry| entry.best_move);

        if USE_SYZYGY && board.occupied_squares.count_ones() <= 5 {
            let result = self.probe_syzygy_root(board);
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
                        pyrrhic_rs::WdlProbeResult::Loss => -MAX_EVAL + ply as i32,
                        pyrrhic_rs::WdlProbeResult::BlessedLoss => -MAX_EVAL + 10000 + ply as i32,
                        pyrrhic_rs::WdlProbeResult::Draw => 0,
                        pyrrhic_rs::WdlProbeResult::CursedWin => MAX_EVAL - 10000 - ply as i32,
                        pyrrhic_rs::WdlProbeResult::Win => MAX_EVAL - ply as i32,
                    };
                }
            }
        }

        //let static_eval = evaluate(board);
        //let improving = ply >= 2 && static_eval > self.previous_static_eval;
        //self.previous_static_eval = static_eval;

        if USE_NULL_MOVE {
            let reduction = 2;
            if !IS_NULL && !board.in_check() && depth > reduction + 1 {
                board.make_null_move();
                let null_move_eval = -self.pvs::<{ NodeType::NonPV as u8 }, true>(board, depth - 1 - reduction, -beta, -beta + 1, ply + 1);
                board.unmake_null_move();

                if null_move_eval >= beta {
                    return null_move_eval; // Beta cutoff, prune this node
                }
            }
        }

        //let futility_margin = 100 * depth as i32;

        let mut moves = generate_moves(board);

        // Check for terminal positions
        // TODO: Remove this if by setting score to mate score
        if moves.is_empty() {
            let king_square = board.piece_squares[Piece::new(PieceType::King, board.side) as usize].lsb();
            return if board.attacked(king_square) {
                -MAX_EVAL + ply as i32
            } else {
                0 // Stalemate
            };
        }

        if USE_MOVE_ORDERING {
            order_moves(self, board, &mut moves, ply, hash_move);
        }

        let mut best_move = None;
        let mut best_eval = -MAX_EVAL + ply as i32;
        let mut evaluation_bound = Bound::Upper;

        // Principal Variation Search
        for (i, &mov) in moves.iter().enumerate() {
            // let evading_check = board.in_check();

            board.make_move(mov);

            let gives_check = board.in_check();

            let mut extensions = 0;

            let do_lmr = !gives_check && !self.is_killer_move(mov, ply) && !board.is_capture(mov);
            //let do_futility = board.is_capture(mov) && static_eval + futility_margin < alpha;

            if gives_check {
                extensions += 1;
            }

            let mut eval;

            if i == 0 || !USE_PVS {
                eval = -self.pvs::<{ NodeType::PV as u8 }, false>(board, depth - 1 + extensions, -beta, -alpha, ply + 1);
            } else if USE_LMR && !do_lmr && i >= 4 && depth >= 3 {
                let reduction = (if board.is_capture(mov) || mov.is_promotion() {
                    0.20 + f64::ln(depth as f64) * f64::ln((i + 1) as f64) / 3.35
                } else {
                    1.35 + f64::ln(depth as f64) * f64::ln((i + 1) as f64) / 2.75
                } as u32);
                //let reduction = 1;
                // Improved LMR formula with better tuned parameters
                //let is_tactical = board.is_capture(mov) || mov.is_promotion();

                //// Base reduction calculation
                //let base = if is_tactical { 0.5 } else { 1.0 };

                //// Calculate dynamic reduction based on depth and move index
                //// Lower divisor for tactical moves to reduce less
                //let divisor = if is_tactical { 3.5 } else { 2.25 };
                //let reduction = (base + (depth as f64).ln() * (i as f64).ln() / divisor).max(1.0) as u32;

                // Ensure we don't reduce too much
                let reduced_depth = (depth - 1 - reduction).max(1);

                eval = -self.pvs::<{ NodeType::NonPV as u8 }, false>(board, reduced_depth, -(alpha + 1), -alpha, ply + 1);

                if eval > alpha {
                    eval = -self.pvs::<{ NodeType::NonPV as u8 }, false>(board, depth - 1 + extensions, -(alpha + 1), -alpha, ply + 1);

                    if on_pv && eval > alpha && eval < beta {
                        eval = -self.pvs::<{ NodeType::PV as u8 }, false>(board, depth - 1 + extensions, -beta, -alpha, ply + 1);
                    }
                }
            } else {
                // Non-PV nodes - scout with null window first
                eval = -self.pvs::<{ NodeType::NonPV as u8 }, false>(board, depth - 1 + extensions, -(alpha + 1), -alpha, ply + 1);

                // Re-search with full window if the move looks promising
                if on_pv && eval > alpha && eval < beta {
                    eval = -self.pvs::<{ NodeType::PV as u8 }, false>(board, depth - 1 + extensions, -beta, -alpha, ply + 1);
                }
            }

            board.unmake_move(mov);

            if self.should_quit(ply) {
                return 0;
            }

            if eval > best_eval {
                best_eval = eval;
                best_move = Some(mov);

                if !USE_ALPHA_BETA || eval > alpha {
                    //if (is_root) {
                    //    self.result.best_move = Some(mov);
                    //}
                    if !IS_NULL {
                        self.pv_table[ply as usize][0] = best_move;

                        let (left, right) = self.pv_table.split_at_mut(ply as usize + 1);

                        if let (Some(dest_row), Some(src_row)) = (left.last_mut(), right.first()) {
                            let dest = &mut dest_row[1..(self.pv_lengths[ply as usize + 1] + 1)];
                            let src = &src_row[0..self.pv_lengths[ply as usize + 1]];
                            dest.copy_from_slice(src);
                        }
                        self.pv_lengths[ply as usize] = self.pv_lengths[ply as usize + 1] + 1;
                    }

                    if USE_ALPHA_BETA {
                        evaluation_bound = Bound::Exact;
                        alpha = eval;
                    }
                }
            }

            if USE_ALPHA_BETA && eval >= beta {
                evaluation_bound = Bound::Lower;

                if !board.is_capture(mov) {
                    self.history[board.side][mov.from()][mov.to()] += depth * depth;
                    self.update_killer_moves(mov, ply);
                }
                break;
            }
        }

        if let Some(best_move) = best_move {
            let entry = TranspositionEntry::new(depth as u8, best_eval as i16, best_move, evaluation_bound, board.zobrist_hash);
            transposition_table::global_store(entry);
        }

        best_eval
    }

    fn quiescence_search(&mut self, board: &mut Board, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        self.result.nodes += 1;

        if self.should_quit(ply) {
            return 0;
        }

        // Currently not required since every move in quiescence is a capture and therefore
        // irreversible
        if board.is_repetition(ply > 2) {
            return 0;
        }

        // Stand pat evaluation
        let stand_pat = evaluate(board);
        if stand_pat >= beta {
            return beta;
        }

        let mut hash_move = None;

        //if USE_TT {
        //    if let Some(entry) = board.transposition_table.probe(board.zobrist_hash) {
        //        if entry.hash == board.zobrist_hash {
        //            hash_move = Some(entry.best_move);
        //            self.result.transpositions += 1;
        //            match entry.node_type {
        //                Bound::Exact => return entry.eval as i32,
        //                Bound::Lower => if entry.eval as i32 >= beta { return entry.eval as i32 },
        //                Bound::Upper => if entry.eval as i32 <= alpha { return entry.eval  as i32 },
        //            }
        //        }
        //    }
        //}

        // Generate and filter captures
        let mut moves = generate_moves(board);
        moves.retain(|mov| board.is_capture(*mov));

        if moves.is_empty() {
            return stand_pat;
        }

        order_moves(self, board, &mut moves, ply, hash_move);

        for mov in moves {
            board.make_move(mov);
            let eval = -self.quiescence_search(board, -beta, -alpha, ply + 1);
            board.unmake_move(mov);

            if self.should_quit(ply) {
                return 0;
            }

            if eval >= beta {
                return beta;
            }
            alpha = alpha.max(eval);
        }

        alpha
    }

    fn probe_syzygy_root(&mut self, board: &Board) -> DtzProbeResult {
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
        //println!("ep dadwa: {}", flip_rank(board.state().en_passant_square.unwrap_or(56)) as u32);
        return self
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
            .expect(&format!("Syzygy tablebase probe failed, fen: {}", board.fen()));
    }
    pub fn extract_pv(&self) -> Vec<Move> {
        let mut pv = Vec::with_capacity(self.pv_lengths[0]);
        for i in 0..self.pv_lengths[0] {
            if let Some(mov) = self.pv_table[0][i] {
                pv.push(mov);
            } else {
                break;
            }
        }
        pv
    }

    fn update_killer_moves(&mut self, mov: Move, ply: u32) {
        let ply = ply as usize;
        if !self.killer_moves[ply].contains(&Some(mov)) {
            self.killer_moves[ply].rotate_right(1);
            self.killer_moves[ply][0] = Some(mov);
        }
    }

    pub fn is_killer_move(&self, mov: Move, ply: u32) -> bool {
        for killer_move in self.killer_moves[ply as usize] {
            if let Some(killer) = killer_move {
                if killer == mov {
                    return true;
                }
            }
        }
        return false;
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

    pub fn print_info(result: &SearchResult) {
        print!(
            "info depth {} score cp {} time {} nodes {} nps {} pv ",
            result.depth_reached,
            result.highest_eval,
            result.time.as_millis(),
            result.nodes,
            ((result.nodes as f64 / result.time.as_nanos() as f64) * 1e9) as u32
        );
        for mov in result.pv.iter() {
            print!("{} ", mov);
        }
        println!();
        println!("tt {}", result.transpositions);
        println!("tt_exact {}", result.transpositions_exact);
        println!("tt_lower {}", result.transpositions_lower);
        println!("tt_upper {}", result.transpositions_upper);
        println!("transpositions {}", result.transpositions);
        println!();
    }
}

impl Default for Search {
    fn default() -> Self {
        Self {
            params: SearchParams::default(),
            result: SearchResult::default(),
            root_ply: 0,
            start_time: Instant::now(),
            pv_table: [[None; MAX_DEPTH]; MAX_DEPTH],
            pv_lengths: [0; MAX_DEPTH],
            max_time: 0,
            killer_moves: [[None; KILLER_MOVE_SLOTS]; MAX_DEPTH],
            should_quit: Arc::new(AtomicBool::new(false)),
            syzygy: pyrrhic_rs::TableBases::<Board>::new("./syzygy/tb345").unwrap(),
            previous_static_eval: 0,
            history: [[[0; 64]; 64]; 2],
        }
    }
}

#[derive(Default, Clone)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub pv: Vec<Move>,
    pub highest_eval: i32,
    pub depth_reached: u32,
    pub nodes: u32,
    pub transpositions_exact: u32,
    pub transpositions_lower: u32,
    pub transpositions_upper: u32,
    pub transpositions: u32,
    pub time: Duration,
}

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
