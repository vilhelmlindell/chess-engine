mod attack_tables;
mod bitboard;
mod board;
mod direction;
mod evaluation;
mod magic_numbers;
mod move_generation;
mod move_ordering;
mod perft;
mod piece;
mod piece_move;
mod piece_square_tables;
mod search;
mod uci;

use board::Board;
use move_generation::*;
use perft::perft;
use std::env;
use uci::Uci;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    //let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    //println!("{}", board.fen());
    Uci::start();
}
