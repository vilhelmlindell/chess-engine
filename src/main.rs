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

use board::Board;
use eframe::{run_native, App, NativeOptions};
use egui::CentralPanel;

pub struct ChessApp;

impl App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello World");
            ui.label("Hello banana man");
        });
    }
}

fn main() {
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R");

    let app = ChessApp;
    let win_option = NativeOptions::default();
    run_native("Chess", win_option, Box::new(|cc| Box::new(app))).unwrap();
}
