pub mod board;
pub mod evaluation;
pub mod move_generation;
pub mod perft;
pub mod search;
pub mod uci;

use uci::Uci;

fn main() {
    Uci::start();
}
