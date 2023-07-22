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

use std::env;
use uci::Uci;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    Uci::start();
}
