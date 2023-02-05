mod bitboard;
mod board;
mod direction;
mod evaluation;
mod magic_numbers;
mod move_generation;
mod piece;
mod piece_move;
mod search;
mod tables;
mod uci;

use bitboard::Bitboard;
use board::Board;

fn main() {
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/Q7/8/PPPPPPPP/RNBQKBNR");
    let moves = board.generate_moves();
    println!("{}", Bitboard(0x000000000000000E));
}
