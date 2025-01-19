use crate::board::piece::{Piece, PieceType};
use crate::board::{Board, Side, TOTAL_MATERIAL_STARTPOS};
use crate::evaluation::piece_square_tables::*;

//const TOTAL_PHASE: i32 = 16 * PieceType::Pawn.phase() + 4 * PieceType::Knight.phase() + 4 * PieceType::Bishop.phase() + 4 * PieceType::Rook.phase() + 2 * PieceType::Queen.phase();

pub fn evaluate(board: &Board) -> i32 {
    //let phase = phase(board);
    //let phase = 0.5;
    let phase = (board.total_material as f32) / (TOTAL_MATERIAL_STARTPOS as f32);
    let mut eval = (board.material_balance * board.side.factor()) as f32;
    eval += ((board.midgame_position_balance * board.side.factor()) as f32) * phase;
    eval += ((board.endgame_position_balance * board.side.factor()) as f32) * (1.0 - phase);
    eval += corner_king_evaluation(board) as f32 * (1.0 - phase*phase) * 10.0;
    //println!("corner_king: {}", corner_king_evaluation(board) as f32 * (1.0 - phase) * 10.0);
    eval as i32
}
//fn phase(board: &Board) -> f32 {
//    let mut phase = 0;
//    for piece_type in [PieceType::Pawn, PieceType::Knight, PieceType::Rook, PieceType::Queen] {
//        phase += piece_type.phase() * board.piece_squares[Piece::new(piece_type, Side::White)].count_ones() as i32;
//        phase += piece_type.phase() * board.piece_squares[Piece::new(piece_type, Side::Black)].count_ones() as i32;
//    }
//    (phase as f32) / (TOTAL_PHASE as f32)
//}
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
