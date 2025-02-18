pub type Square = usize;

pub fn flip_rank(square: Square) -> Square {
    let rank = square / 8;
    let file = square % 8;
    let new_rank = 7 - rank;
    new_rank * 8 + file
}
