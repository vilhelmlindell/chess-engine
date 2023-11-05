pub mod board;
pub mod evaluation;
pub mod move_generation;
pub mod perft;
pub mod search;
pub mod uci;

use std::env;
use uci::Uci;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    Uci::start();
}
