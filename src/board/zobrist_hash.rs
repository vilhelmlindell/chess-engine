use crate::board::{Board, Side};
use rand::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use ctor::ctor;

use super::piece::Piece;
use super::piece_move::Square;

const NUM_SQUARES: usize = 64;
const NUM_PIECES: usize = 12;
const NUM_CASTLING_RIGHTS: usize = 16;
const NUM_EN_PASSANT_SQUARES: usize = 8;
const ZOBRIST_SEED: u64 = 0x123456789ABCDEF;

pub static mut ZOBRIST_SQUARES: [[u64; NUM_PIECES]; NUM_SQUARES] = [[0; NUM_PIECES]; NUM_SQUARES];
pub static mut ZOBRIST_SIDE: u64 = 0;
pub static mut ZOBRIST_CASTLING_RIGHTS: [u64; NUM_CASTLING_RIGHTS] = [0; NUM_CASTLING_RIGHTS];
pub static mut ZOBRIST_EN_PASSANT_SQUARE: [u64; NUM_EN_PASSANT_SQUARES] = [0; NUM_EN_PASSANT_SQUARES];

#[ctor]
pub fn initialize_zobrist_tables() {
    let mut rng = ChaCha8Rng::seed_from_u64(ZOBRIST_SEED);
    unsafe {
        ZOBRIST_SQUARES = precompute_zobrist_squares(&mut rng);
        ZOBRIST_SIDE = precompute_zobrist_side(&mut rng);
        ZOBRIST_CASTLING_RIGHTS = precompute_zobrist_castling_rights(&mut rng);
        ZOBRIST_EN_PASSANT_SQUARE = precompute_zobrist_en_passant_square(&mut rng);
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

fn precompute_zobrist_squares(rng: &mut ChaCha8Rng) -> [[u64; NUM_PIECES]; NUM_SQUARES] {
    let mut zobrist_hash = [[0; NUM_PIECES]; NUM_SQUARES];
    for i in 0..NUM_SQUARES {
        for j in 0..NUM_PIECES {
            zobrist_hash[i][j] = rng.gen();
        }
    }
    zobrist_hash
}

fn precompute_zobrist_side(rng: &mut ChaCha8Rng) -> u64 {
    rng.gen()
}

fn precompute_zobrist_castling_rights(rng: &mut ChaCha8Rng) -> [u64; NUM_CASTLING_RIGHTS] {
    let mut zobrist_castling_rights = [0; NUM_CASTLING_RIGHTS];
    for i in 0..NUM_CASTLING_RIGHTS {
        zobrist_castling_rights[i] = rng.gen();
    }
    zobrist_castling_rights
}

pub fn precompute_zobrist_en_passant_square(rng: &mut ChaCha8Rng) -> [u64; NUM_EN_PASSANT_SQUARES] {
    let mut zobrist_en_passant_square = [0; NUM_EN_PASSANT_SQUARES];
    for i in 0..NUM_EN_PASSANT_SQUARES {
        zobrist_en_passant_square[i] = rng.gen();
    }
    zobrist_en_passant_square
}

#[cfg(test)]
mod test {
    use crate::board::piece_move::{Move, MoveType};

    use super::*;

    #[test]
    fn transposition() {
        let mut board1 = Board::start_pos();
        let mut board2 = Board::start_pos();

        let b1b3 = Move::new(57, 42, MoveType::Normal);
        let g1f3 = Move::new(62, 45, MoveType::Normal);

        let b8c6 = Move::new(1, 18, MoveType::Normal);
        let g8f6 = Move::new(6, 21, MoveType::Normal);

        assert_eq!(board1.zobrist_hash, board2.zobrist_hash);

        board1.make_move(b1b3);
        board1.make_move(b8c6);
        board1.make_move(g1f3);
        board1.make_move(g8f6);

        board2.make_move(g1f3);
        board2.make_move(g8f6);
        board2.make_move(b1b3);
        board2.make_move(b8c6);

        assert_eq!(board1.zobrist_hash, board2.zobrist_hash);

        board1.unmake_move(b8c6);
        board1.unmake_move(b1b3);

        board2.unmake_move(b1b3);
        board2.unmake_move(b8c6);

        assert_eq!(board1.zobrist_hash, board2.zobrist_hash);
    }
}
