use crate::board::piece::{Piece, PieceType};
use crate::board::Board;
use crate::evaluation::piece_square_tables::*;

pub fn evaluate(board: &Board) -> i32 {
    let endgame_weight = 1.0 - (board.side_squares[board.side.enemy()].count_ones() as f32 / 16.0);
    let mut eval = (board.material_balance * board.side.factor()) as f32;
    eval += (board.position_balance * board.side.factor()) as f32;
    eval += corner_king_evaluation(board) as f32 * endgame_weight * endgame_weight * endgame_weight * 15.0;
    eval as i32
}
fn corner_king_evaluation(board: &Board) -> i32 {
    let friendly_king_square = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();
    let enemy_king_square = board.piece_squares[Piece::new(PieceType::King, board.side.enemy())].lsb();
    let enemy_king_center_distance = CENTER_DISTANCE_TABLE[enemy_king_square];

    let mut eval = enemy_king_center_distance;

    let enemy_king_file = (enemy_king_square % 8) as i32;
    let enemy_king_rank = (enemy_king_square / 8) as i32;

    let friendly_king_file = (friendly_king_square % 8) as i32;
    let friendly_king_rank = (friendly_king_square / 8) as i32;

    let king_distance = i32::abs(enemy_king_file - friendly_king_file) + i32::abs(enemy_king_rank - friendly_king_rank);

    eval += 14 - king_distance;
    eval
}
