mod bitboard;
mod board;
mod evaluation;
mod magic_numbers;
mod r#move;
mod move_generation;
mod piece;
mod tables;

use board::Board;
use tables::KNIGHT_ATTACK_MASKS;

fn main() {
    let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
    println!("{}", board);
}
