#[rustfmt::skip]
mod unformatted {
    use crate::board::Side;
    use crate::board::piece::PieceType;

    const MIDGAME_PAWN_SQUARE_TABLE: [i32; 64] = [
         0,   0,   0,   0,   0,   0,   0,   0,
        50,  50,  50,  50,  50,  50,  50,  50,
        10,  10,  20,  30,  30,  20,  10,  10,
         5,   5,  10,  25,  25,  10,   5,   5,
         0,   0,   0,  20,  20,   0,   0,   0,
         5,  -5, -10,   0,   0, -10,  -5,   5,
         5,  10,  10, -20, -20,  10,  10,   5,
         0,   0,   0,   0,   0,   0,   0,   0,
    ];
    const MIDGAME_KNIGHT_SQUARE_TABLE: [i32; 64] = [
        -50, -40, -30, -30, -30, -30, -40, -50,
        -40, -20,   0,   0,   0,   0, -20, -40,
        -30,   0,  10,  15,  15,  10,   0, -30,
        -30,   5,  15,  20,  20,  15,   5, -30,
        -30,   0,  15,  20,  20,  15,   0, -30,
        -30,   5,  10,  15,  15,  10,   5, -30,
        -40, -20,   0,   5,   5,   0, -20, -40,
        -50, -40, -30, -30, -30, -30, -40, -50,
    ];
    const MIDGAME_BISHOP_SQUARE_TABLE: [i32; 64] = [
        -20, -10, -10, -10, -10, -10, -10, -20,
        -10,   0,   0,   0,   0,   0,   0, -10,
        -10,   0,   5,  10,  10,   5,   0, -10,
        -10,   5,   5,  10,  10,   5,   5, -10,
        -10,   0,  10,  10,  10,  10,   0, -10,
        -10,  10,  10,  10,  10,  10,  10, -10,
        -10,   5,   0,   0,   0,   0,   5, -10,
        -20, -10, -10, -10, -10, -10, -10, -20,
    ];
    const MIDGAME_ROOK_SQUARE_TABLE: [i32; 64] = [
         0,   0,   0,   0,   0,   0,   0,   0,
         5,  10,  10,  10,  10,  10,  10,   5,
        -5,   0,   0,   0,   0,   0,   0,  -5,
        -5,   0,   0,   0,   0,   0,   0,  -5,
        -5,   0,   0,   0,   0,   0,   0,  -5,
        -5,   0,   0,   0,   0,   0,   0,  -5,
        -5,   0,   0,   0,   0,   0,   0,  -5,
         0,   0,   0,   5,   5,   0,   0,   0,
    ];
    const MIDGAME_QUEEN_SQUARE_TABLE: [i32; 64] = [
        -20, -10, -10,  -5,  -5, -10, -10, -20,
        -10,   0,   0,   0,   0,   0,   0, -10,
        -10,   0,   5,   5,   5,   5,   0, -10,
         -5,   0,   5,   5,   5,   5,   0,  -5,
          0,   0,   5,   5,   5,   5,   0,  -5,
        -10,   5,   5,   5,   5,   5,   0, -10,
        -10,   0,   5,   0,   0,   0,   0, -10,
        -20, -10, -10,  -5,  -5, -10, -10, -20,
    ];
    const MIDGAME_KING_SQUARE_TABLE: [i32; 64] = [
        -80, -70, -70, -70, -70, -70, -70, -80,
        -60, -60, -60, -60, -60, -60, -60, -60,
        -40, -50, -50, -60, -60, -50, -50, -40,
        -30, -40, -40, -50, -50, -40, -40, -30,
        -20, -30, -30, -40, -40, -30, -30, -20,
        -10, -20, -20, -20, -20, -20, -20, -10,
         20,  20,  -5,  -5,  -5,  -5,  20,  20,
         20,  30,  10,   0,   0,  10,  30,  20,
    ];

    const ENDGAME_PAWN_SQUARE_TABLE: [i32; 64] = [
         0,   0,   0,   0,   0,   0,   0,   0,
        80,  80,  80,  80,  80,  80,  80,  80,
        50,  50,  50,  50,  50,  50,  50,  50,
        30,  30,  30,  30,  30,  30,  30,  30,
        20,  20,  20,  20,  20,  20,  20,  20,
        10,  10,  10,  10,  10,  10,  10,  10,
        10,  10,  10,  10,  10,  10,  10,  10,
         0,   0,   0,   0,   0,   0,   0,   0,
    ];
    const ENDGAME_KNIGHT_SQUARE_TABLE: [i32; 64] = [
        -50, -40, -30, -30, -30, -30, -40, -50,
        -40, -20,   0,   0,   0,   0, -20, -40,
        -30,   0,  10,  15,  15,  10,   0, -30,
        -30,   5,  15,  20,  20,  15,   5, -30,
        -30,   0,  15,  20,  20,  15,   0, -30,
        -30,   5,  10,  15,  15,  10,   5, -30,
        -40, -20,   0,   5,   5,   0, -20, -40,
        -50, -40, -30, -30, -30, -30, -40, -50,
    ];
    const ENDGAME_BISHOP_SQUARE_TABLE: [i32; 64] = [
        -20, -10, -10, -10, -10, -10, -10, -20,
        -10,   0,   0,   0,   0,   0,   0, -10,
        -10,   0,   5,  10,  10,   5,   0, -10,
        -10,   5,   5,  10,  10,   5,   5, -10,
        -10,   0,  10,  10,  10,  10,   0, -10,
        -10,  10,  10,  10,  10,  10,  10, -10,
        -10,   5,   0,   0,   0,   0,   5, -10,
        -20, -10, -10, -10, -10, -10, -10, -20,
    ];
    const ENDGAME_ROOK_SQUARE_TABLE: [i32; 64] = [
         0,   0,   0,   0,   0,   0,   0,   0,
         5,  10,  10,  10,  10,  10,  10,   5,
        -5,   0,   0,   0,   0,   0,   0,  -5,
        -5,   0,   0,   0,   0,   0,   0,  -5,
        -5,   0,   0,   0,   0,   0,   0,  -5,
        -5,   0,   0,   0,   0,   0,   0,  -5,
        -5,   0,   0,   0,   0,   0,   0,  -5,
         0,   0,   0,   5,   5,   0,   0,   0,
    ];
    const ENDGAME_QUEEN_SQUARE_TABLE: [i32; 64] = [
        -20, -10, -10,  -5,  -5, -10, -10, -20,
        -10,   0,   0,   0,   0,   0,   0, -10,
        -10,   0,   5,   5,   5,   5,   0, -10,
         -5,   0,   5,   5,   5,   5,   0,  -5,
          0,   0,   5,   5,   5,   5,   0,  -5,
        -10,   5,   5,   5,   5,   5,   0, -10,
        -10,   0,   5,   0,   0,   0,   0, -10,
        -20, -10, -10,  -5,  -5, -10, -10, -20,
    ];
    const ENDGAME_KING_SQUARE_TABLE: [i32; 64] = [
        -20, -10, -10, -10, -10, -10, -10, -20,
         -5,   0,   5,   5,   5,   5,   0,  -5,
        -10,  -5,  20,  30,  30,  20,  -5, -10,
        -15, -10,  35,  45,  45,  35, -10, -15,
        -20, -15,  30,  40,  40,  30, -15, -20,
        -25, -20,  20,  25,  25,  20, -20, -25,
        -30, -25,   0,   0,   0,   0, -25, -30,
        -50, -30, -30, -30, -30, -30, -30, -50,
    ];

    pub const CENTER_DISTANCE_TABLE: [i32; 64] = [
         6, 5, 4, 3, 3, 4, 5, 6,
         5, 4, 3, 2, 2, 3, 4, 5,
         4, 3, 2, 1, 1, 2, 3, 4,
         3, 2, 1, 0, 0, 1, 2, 3,
         3, 2, 1, 0, 0, 1, 2, 3,
         4, 3, 2, 1, 1, 2, 3, 4,
         5, 4, 3, 2, 2, 3, 4, 5,
         6, 5, 4, 3, 3, 4, 5, 6
    ];

    const MIDGAME_PIECE_SQUARE_TABLES: [[i32; 64]; 6] = [MIDGAME_PAWN_SQUARE_TABLE, MIDGAME_KNIGHT_SQUARE_TABLE, MIDGAME_BISHOP_SQUARE_TABLE, MIDGAME_ROOK_SQUARE_TABLE, MIDGAME_QUEEN_SQUARE_TABLE, MIDGAME_KING_SQUARE_TABLE];
    const ENDGAME_PIECE_SQUARE_TABLES: [[i32; 64]; 6] = [ENDGAME_PAWN_SQUARE_TABLE, ENDGAME_KNIGHT_SQUARE_TABLE, ENDGAME_BISHOP_SQUARE_TABLE, ENDGAME_ROOK_SQUARE_TABLE, ENDGAME_QUEEN_SQUARE_TABLE, ENDGAME_KING_SQUARE_TABLE];

    pub fn midgame_position_value(piece_type: PieceType, square: usize, side: Side) -> i32 {
        match side {
            Side::White => MIDGAME_PIECE_SQUARE_TABLES[piece_type][square],
            Side::Black => {
                let rank = square / 8;
                let file = square % 8;
                let actual_rank = 7 - rank;
                let new_square = file + actual_rank * 8;
                MIDGAME_PIECE_SQUARE_TABLES[piece_type][new_square as usize]
          }
        }
    }
    pub fn endgame_position_value(piece_type: PieceType, square: usize, side: Side) -> i32 {
        match side {
            Side::White => ENDGAME_PIECE_SQUARE_TABLES[piece_type][square],
            Side::Black => {
                let rank = square / 8;
                let file = square % 8;
                let actual_rank = 7 - rank;
                let new_square = file + actual_rank * 8;
                ENDGAME_PIECE_SQUARE_TABLES[piece_type][new_square as usize]
            }
        }
    }
}

pub use unformatted::*;
