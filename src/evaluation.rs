use crate::board::Board;
use crate::piece_square_tables::PIECE_SQUARE_TABLES;

impl Board {
    pub fn evaluate(&self) -> i32 {
        self.material_balance
    }
}
