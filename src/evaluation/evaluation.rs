use crate::board::piece::{Piece, PieceType};
use crate::board::Board;
use crate::evaluation::piece_square_tables::*;

impl Board {
    pub fn evaluate(&self) -> i32 {
        let endgame_weight = 1.0 - (self.side_squares[self.side.enemy()].count_ones() as f32 / 16.0);
        let mut eval = self.material_balance * self.side.factor();
        eval += ((1.0 - endgame_weight) * (self.position_balance * self.side.factor()) as f32) as i32;
        eval += self.corner_king_evaluation(endgame_weight);
        eval
    }
    fn corner_king_evaluation(&self, endgame_weight: f32) -> i32 {
        let friendly_king_square = self.piece_squares[Piece::new(PieceType::King, self.side)].lsb();
        let enemy_king_square = self.piece_squares[Piece::new(PieceType::King, self.side.enemy())].lsb();
        let enemy_king_center_distance = CENTER_DISTANCE_TABLE[enemy_king_square];

        let mut eval = enemy_king_center_distance;

        let enemy_king_file = (enemy_king_square % 8) as i32;
        let enemy_king_rank = (enemy_king_square / 8) as i32;

        let friendly_king_file = (friendly_king_square % 8) as i32;
        let friendly_king_rank = (friendly_king_square / 8) as i32;

        let king_distance = i32::abs(enemy_king_file - friendly_king_file) + i32::abs(enemy_king_rank - friendly_king_rank);

        eval += 14 - king_distance;
        (eval as f32 * endgame_weight * 10.0) as i32
    }
}
