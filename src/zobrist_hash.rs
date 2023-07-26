use crate::board::{Board, Side};
use once_cell::sync::Lazy;
use rand::*;

const NUM_SQUARES: usize = 64;
const NUM_PIECES: usize = 12;
const NUM_CASTLING_RIGHTS: usize = 16;
const NUM_EN_PASSANT_SQUARES: usize = 8;

pub static ZOBRIST_SQUARES: Lazy<[[u64; NUM_PIECES]; NUM_SQUARES]> = Lazy::new(initialize_zobrist_squares);
pub static ZOBRIST_SIDE_TO_MOVE: Lazy<u64> = Lazy::new(initialize_zobrist_side_to_move);
pub static ZOBRIST_CASTLING_RIGHTS: Lazy<[u64; NUM_CASTLING_RIGHTS]> = Lazy::new(initialize_zobrist_castling_rights);
pub static ZOBRIST_EN_PASSANT_SQUARE: Lazy<[u64; NUM_EN_PASSANT_SQUARES]> = Lazy::new(initialize_zobrist_en_passant_square);

pub fn get_zobrist_hash(board: &Board) -> u64 {
    let mut zobrist_hash = 0;
    for square in 0..NUM_SQUARES {
        if let Some(piece) = board.squares[square] {
            zobrist_hash ^= ZOBRIST_SQUARES[square][piece as usize];
        }
    }
    if board.side == Side::White {
        zobrist_hash ^= *ZOBRIST_SIDE_TO_MOVE;
    }
    let white_castling = board.state().castling_rights[Side::White];
    let black_castling = board.state().castling_rights[Side::Black];
    let castling_rights_combined = (white_castling.kingside as usize) << 3 | (white_castling.queenside as usize) << 2 | (black_castling.kingside as usize) << 1 | black_castling.queenside as usize;
    zobrist_hash ^= ZOBRIST_CASTLING_RIGHTS[castling_rights_combined];
    if let Some(square) = board.state().en_passant_square {
        let file = square % 8;
        zobrist_hash ^= ZOBRIST_EN_PASSANT_SQUARE[file];
    }
    zobrist_hash
}

fn initialize_zobrist_squares() -> [[u64; NUM_PIECES]; NUM_SQUARES] {
    let mut rng = thread_rng();
    let mut zobrist_hash = [[0; NUM_PIECES]; NUM_SQUARES];
    for i in 0..NUM_SQUARES {
        for j in 0..NUM_PIECES {
            zobrist_hash[i][j] = rng.gen();
        }
    }
    zobrist_hash
}

fn initialize_zobrist_side_to_move() -> u64 {
    let mut rng = thread_rng();
    rng.gen()
}

fn initialize_zobrist_castling_rights() -> [u64; NUM_CASTLING_RIGHTS] {
    let mut rng = thread_rng();
    let mut zobrist_castling_rights = [0; NUM_CASTLING_RIGHTS];
    for i in 0..NUM_CASTLING_RIGHTS {
        zobrist_castling_rights[i] = rng.gen();
    }
    zobrist_castling_rights
}

fn initialize_zobrist_en_passant_square() -> [u64; NUM_EN_PASSANT_SQUARES] {
    let mut rng = thread_rng();
    let mut zobrist_en_passant_square = [0; NUM_EN_PASSANT_SQUARES];
    for i in 0..NUM_EN_PASSANT_SQUARES {
        zobrist_en_passant_square[i] = rng.gen();
    }
    zobrist_en_passant_square
}
