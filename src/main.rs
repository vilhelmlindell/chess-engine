mod bitboard;
mod board;
mod direction;
mod evaluation;
mod magic_numbers;
mod r#move;
mod move_generation;
mod piece;
mod tables;
mod uci;

use board::Board;

fn main() {
    let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/Q7/8/PPPPPPPP/RNBQKBNR");
    let moves = board.generate_moves();
    println!("{}", board);
    println!("{}", moves.len());
}
