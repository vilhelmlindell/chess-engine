use crate::board::{piece::PieceType, piece_move::Move, Board};

use super::Search;

// Define an enum for move categories with explicit ordering
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
enum MoveCategory {
    PvMove = 7,
    HashMove = 6,
    WinningCapture = 5,
    EqualCapture = 4,
    Promotion = 3,
    KillerMove = 2,
    QuietMove = 1,
    LosingCapture = 0,
}

pub fn order_moves(search: &Search, board: &Board, moves: &mut [Move], ply: u32, hash_move: Option<Move>) {
    moves.sort_by_cached_key(|mov| {
        let (category, score) = categorize_move(search, *mov, board, ply, hash_move);
        // Use tuple for sorting: primary sort by category (high to low), secondary by score within category
        (-(category as i32), -score)
    });
}

fn categorize_move(search: &Search, mov: Move, board: &Board, ply: u32, hash_move: Option<Move>) -> (MoveCategory, i32) {
    // Check for PV move
    if Some(mov) == search.result.pv.get(ply as usize).cloned() {
        return (MoveCategory::PvMove, 0);
    }

    // Check for hash move
    if Some(mov) == hash_move {
        return (MoveCategory::HashMove, 0);
    }

    // Handle captures
    if board.is_capture(mov) {
        let captured_piece = board.squares[mov.to()].unwrap();
        let moving_piece = board.squares[mov.from()].unwrap();
        let capture_score = captured_piece.piece_type().centipawns() - moving_piece.piece_type().centipawns();

        if capture_score > 0 {
            return (MoveCategory::WinningCapture, capture_score);
        } else if capture_score == 0 {
            return (MoveCategory::EqualCapture, 0);
        } else {
            return (MoveCategory::LosingCapture, capture_score);
        }
    }

    // Handle promotions
    if let Some(piece) = mov.move_type().promotion_piece() {
        let promotion_score = piece.centipawns() - PieceType::Pawn.centipawns();
        return (MoveCategory::Promotion, promotion_score);
    }

    // Handle killer moves
    if search.is_killer_move(mov, ply) {
        return (MoveCategory::KillerMove, 0);
    }

    // Handle quiet moves with history score
    let history_score = search.history[board.side][mov.from()][mov.to()] as i32;
    (MoveCategory::QuietMove, history_score)
}

// Original move_score kept for compatibility if needed elsewhere
fn move_score(search: &mut Search, mov: Move, board: &Board, ply: u32, hash_move: Option<Move>) -> i32 {
    let (category, score) = categorize_move(search, mov, board, ply, hash_move);

    // Combine category and score for a single integer value
    // Category is the primary component (shifted left), score is secondary
    ((category as i32) << 24) + score
}
