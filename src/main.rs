mod app;
mod bitboard;
mod board;
mod direction;
mod evaluation;
mod magic_numbers;
mod move_generation;
mod piece;
mod piece_move;
mod search;
mod tables;
mod uci;

use app::ChessApp;
use egui::Vec2;

fn main() {
    let mut options = eframe::NativeOptions::default();
    options.resizable = false;
    options.initial_window_size = Option::from(Vec2::new(600.0, 600.0));
    eframe::run_native("Chess", options, Box::new(|cc| Box::new(ChessApp::new(cc)))).unwrap();
}
