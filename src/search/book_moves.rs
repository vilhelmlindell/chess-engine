use crate::board::piece_move::Move;
use crate::board::Board;
use rand::Rng;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::sync::LazyLock;

struct BookMove {
    move_string: String,
    times_played: u32,
}

static MOVES_BY_POSITION: LazyLock<HashMap<String, Vec<BookMove>>> = LazyLock::new(initialize_book_moves);

pub fn get_book_move(board: &Board, times_played_weight: f32) -> Option<Move> {
    let fen = board.fen().split_whitespace().take(3).collect::<Vec<&str>>().join(" ");
    if let Some(moves) = MOVES_BY_POSITION.get(&fen) {
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
                return Some(Move::from_long_algebraic(&moves.get(index).unwrap().move_string, board));
            }
        }
        return Some(Move::from_long_algebraic(&moves.last().unwrap().move_string, board));
    }
    None
}
fn initialize_book_moves() -> HashMap<String, Vec<BookMove>> {
    let mut moves_by_position: HashMap<String, Vec<BookMove>> = HashMap::new();
    let mut current_position = String::new();

    for line in read_to_string("opening_book.txt").unwrap().lines() {
        if line.starts_with("pos") {
            // Extract the position key.
            current_position = line.chars().skip(4).collect();
            moves_by_position.entry(current_position.clone()).or_default();
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
    moves_by_position
}
