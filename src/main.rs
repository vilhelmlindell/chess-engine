mod board;
mod evaluation;
mod move_generation;
mod perft;
mod search;
mod uci;

use std::env;
use uci::Uci;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    Uci::start();
}
