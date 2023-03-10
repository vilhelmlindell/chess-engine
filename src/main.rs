mod app;
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
use uci::UCI;

fn main() {
    for i in 0..7 {
        println!("Depth: {}, Nodes: {}", i, perft(&i));
    }
    //UCI::start();
}
