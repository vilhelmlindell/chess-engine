use crate::board::piece_move::Move;
use crate::board::Board;
use once_cell::sync::Lazy;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::{collections::HashMap, fs::File};

struct BookMove {
    move_string: String,
    times_played: u32,
}

static MOVES_BY_POSITION: Lazy<HashMap<String, Vec<BookMove>>> = Lazy::new(initialize_book_moves);

pub fn get_book_move(current_fen: &String, times_played_weight: f32) -> Option<Move> {
    if let Some(moves) = MOVES_BY_POSITION.get(current_fen) {
        let weighted_play_count = |play_count: u32| f32::powf(play_count as f32, times_played_weight) as u32;
        let mut weights: Vec<u32> = Vec::new();
        let weight_sum = moves.iter().fold(0, |acc, mov| {
            let weight = weighted_play_count(mov.times_played);
            weights.push(weight);
            acc + weight
        });
        let mut rng = rand::thread_rng();
        let random_number = rng.gen_range(0..=weight_sum);
        let mut acc_weights = 0;
        for (index, weight) in weights.iter().enumerate() {
            acc_weights += weight;
            if acc_weights >= random_number {
                Move::from(value)
            }
        }
    }
    None
}
fn initialize_book_moves() -> HashMap<String, Vec<BookMove>> {
    let mut moves_by_position: HashMap<String, Vec<BookMove>> = HashMap::new();
    let mut current_position = String::new();

    // Open the file and read it line by line.
    if let Ok(file) = File::open("opening_book.txt") {
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                if line.starts_with("pos") {
                    // Extract the position key.
                    current_position = line.split_whitespace().nth(1).unwrap().to_string();
                    moves_by_position.entry(current_position.clone()).or_insert(Vec::new());
                } else {
                    // Parse the move and times played.
                    let mut parts = line.split_whitespace();
                    let move_string = parts.next().unwrap().to_string();
                    let times_played: u32 = parts.next().unwrap().parse().unwrap();

                    // Create a BookMove entry and add it to the hashmap.
                    let entry = BookMove { move_string, times_played };

                    moves_by_position.entry(current_position.clone()).and_modify(|e| e.push(entry));
                }
            }
        }
    } else {
        eprintln!("Error: Unable to open the file 'opening_book.txt'");
    }

    moves_by_position
}
