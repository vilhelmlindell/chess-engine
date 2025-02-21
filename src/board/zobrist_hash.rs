use crate::board::{Board, Side};
use rand::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng; // More efficient than default RNG for our use case
use std::sync::LazyLock;
use ctor::ctor;

use super::piece::Piece;
use super::piece_move::Square;

const NUM_SQUARES: usize = 64;
const NUM_PIECES: usize = 12;
const NUM_CASTLING_RIGHTS: usize = 16;
const NUM_EN_PASSANT_SQUARES: usize = 8;
const ZOBRIST_SEED: u64 = 0x123456789ABCDEF; // You can change this seed value

pub static mut ZOBRIST_SQUARES: [[u64; NUM_PIECES]; NUM_SQUARES] = [[0; NUM_PIECES]; NUM_SQUARES];
pub static mut ZOBRIST_SIDE: u64 = 0;
pub static mut ZOBRIST_CASTLING_RIGHTS: [u64; NUM_CASTLING_RIGHTS] = [0; NUM_CASTLING_RIGHTS];
pub static mut ZOBRIST_EN_PASSANT_SQUARE: [u64; NUM_EN_PASSANT_SQUARES] = [0; NUM_EN_PASSANT_SQUARES];

#[ctor]
pub fn initialize_zobrist_tables() {
    unsafe {
        ZOBRIST_SQUARES = precompute_zobrist_squares();
        ZOBRIST_SIDE = precompute_zobrist_side();
        ZOBRIST_CASTLING_RIGHTS = precompute_zobrist_castling_rights();
        ZOBRIST_EN_PASSANT_SQUARE = precompute_zobrist_en_passant_square();
    }
}

pub fn get_zobrist_squares(square: Square, piece: Piece) -> u64 {
    unsafe { ZOBRIST_SQUARES[square][piece as usize] }
}

pub fn get_zobrist_side() -> u64 {
    unsafe { ZOBRIST_SIDE }
}

pub fn get_zobrist_castling_rights(castling_rights_bits: usize) -> u64 {
    unsafe { ZOBRIST_CASTLING_RIGHTS[castling_rights_bits] }
}

pub fn get_zobrist_en_passant_square(file: usize) -> u64{
    unsafe { ZOBRIST_EN_PASSANT_SQUARE[file] }
}

pub fn get_zobrist_hash(board: &Board) -> u64 {
    unsafe {
        let mut zobrist_hash = 0;
        for square in 0..NUM_SQUARES {
            if let Some(piece) = board.squares[square] {
                zobrist_hash ^= get_zobrist_squares(square, piece);
            }
        }

        if board.side == Side::Black {
            zobrist_hash ^= get_zobrist_side();
        }

        let white_castling = board.state().castling_rights[Side::White];
        let black_castling = board.state().castling_rights[Side::Black];
        let castling_rights_combined = (white_castling.kingside as usize) << 3 | (white_castling.queenside as usize) << 2 | (black_castling.kingside as usize) << 1 | black_castling.queenside as usize;
        zobrist_hash ^= get_zobrist_castling_rights(castling_rights_combined);

        if let Some(square) = board.state().en_passant_square {
            let file = square % 8;
            zobrist_hash ^= get_zobrist_en_passant_square(file);
        }

        zobrist_hash
    }
}

fn get_seeded_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(ZOBRIST_SEED)
}

fn precompute_zobrist_squares() -> [[u64; NUM_PIECES]; NUM_SQUARES] {
    let mut rng = get_seeded_rng();
    let mut zobrist_hash = [[0; NUM_PIECES]; NUM_SQUARES];
    for i in 0..NUM_SQUARES {
        for j in 0..NUM_PIECES {
            zobrist_hash[i][j] = rng.gen();
        }
    }
    zobrist_hash
}

fn precompute_zobrist_side() -> u64 {
    let mut rng = get_seeded_rng();
    rng.gen()
}

fn precompute_zobrist_castling_rights() -> [u64; NUM_CASTLING_RIGHTS] {
    let mut rng = get_seeded_rng();
    let mut zobrist_castling_rights = [0; NUM_CASTLING_RIGHTS];
    for i in 0..NUM_CASTLING_RIGHTS {
        zobrist_castling_rights[i] = rng.gen();
    }
    zobrist_castling_rights
}

pub fn precompute_zobrist_en_passant_square() -> [u64; NUM_EN_PASSANT_SQUARES] {
    let mut rng = get_seeded_rng();
    let mut zobrist_en_passant_square = [0; NUM_EN_PASSANT_SQUARES];
    for i in 0..NUM_EN_PASSANT_SQUARES {
        zobrist_en_passant_square[i] = rng.gen();
    }
    zobrist_en_passant_square
}
