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

use perft::perft;
use std::env;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    for i in 1..=1 {
        println!("Depth: {}", i);
        println!("{}", perft(i));
    }
    //UCI::start();
}
