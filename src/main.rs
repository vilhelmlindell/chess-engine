mod app;
mod bitboard;
mod board;
mod direction;
mod evaluation;
mod magic_numbers;
mod move_generation;
mod move_ordering;
mod piece;
mod piece_move;
mod piece_square_tables;
mod search;
mod tables;
mod uci;

use board::Board;
use std::env;
use uci::UCI;

use crate::{board::Side, piece::Piece};

fn main() {
    UCI::start();
    //let mut options = eframe::NativeOptions::default();
    //options.resizable = false;
    //options.initial_window_size = Option::from(Vec2::new(1200.0, 1200.0));
    //eframe::run_native("Chess", options, Box::new(|cc| Box::new(ChessApp::new(cc)))).unwrap();
}
