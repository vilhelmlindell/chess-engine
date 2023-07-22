use crate::board::Board;

impl Board {
    pub fn evaluate(&self) -> i32 {
        self.material_balance + self.position_balance
        //self.material_balance
    }
}
