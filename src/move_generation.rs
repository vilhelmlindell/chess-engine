use crate::bitboard::*;
use crate::board::{Board, Side};
use crate::piece::{Piece, PieceType};
use crate::r#move::Move;
use crate::tables::KNIGHT_ATTACK_MASKS;

pub fn generate_pawn_moves(board: &Board) -> Vec<Move> {
    let up: &Direction = match board.side_to_move {
        Side::White => &Direction::North,
        Side::Black => &Direction::South,
    };
    let up_left: &Direction = match board.side_to_move {
        Side::White => &Direction::NorthWest,
        Side::Black => &Direction::SouthEast,
    };
    let up_right: &Direction = match board.side_to_move {
        Side::White => &Direction::NorthEast,
        Side::Black => &Direction::SouthWest,
    };
    let mut moves = Vec::<Move>::with_capacity(16);
    let bitboard = board
        .piece_bitboards
        .get(&Piece::new(PieceType::Pawn, board.side_to_move))
        .unwrap()
        .clone();

    let mut pushed_pawns = push_pawns(&bitboard, &!board.occupied_squares, &board.side_to_move);
    let mut double_pushed_pawns =
        push_pawns(&bitboard, &!board.occupied_squares, &board.side_to_move);

    while pushed_pawns != 0 {
        let square = pushed_pawns.pop_lsb();
        moves.push(Move::new((square as i32 - up.value()) as u32, square));
    }
    while double_pushed_pawns != 0 {
        let square = double_pushed_pawns.pop_lsb();
        moves.push(Move::new((square as i32 - up.value() * 2) as u32, square));
    }
    let mut capturing_pawns_up_left = bitboard.shift(up_left) & board.enemy_squares();
    let mut capturing_pawns_up_right = bitboard.shift(up_right) & board.enemy_squares();

    while capturing_pawns_up_left != 0 {
        let square = capturing_pawns_up_left.pop_lsb();
        moves.push(Move::new((square as i32 - up_left.value()) as u32, square));
    }
    while capturing_pawns_up_right != 0 {
        let square = capturing_pawns_up_right.pop_lsb();
        moves.push(Move::new((square as i32 - up_right.value()) as u32, square));
    }
    moves
}

fn push_pawns(pawns: &Bitboard, empty_squares: &Bitboard, side_to_move: &Side) -> Bitboard {
    (pawns.north() >> ((side_to_move.value()) << 4)) & *empty_squares
}

pub fn generate_knight_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::<Move>::with_capacity(96);
    let mut bitboard = board
        .piece_bitboards
        .get(&Piece::new(PieceType::Knight, board.side_to_move))
        .unwrap()
        .clone();

    while bitboard != 0 {
        let square = bitboard.pop_lsb();
        let mut attack_bitboard =
            KNIGHT_ATTACK_MASKS[square as usize].clone() & !board.friendly_squares();
        while attack_bitboard != 0 {
            let end_square = attack_bitboard.pop_lsb();
            moves.push(Move::new(square, end_square));
        }
    }
    moves
}

//pub fn generate_bishop_moves(board: &Board) -> Vec<Move> {}
//pub fn generate_rook_moves(board: &Board) -> Vec<Move> {}
//pub fn generate_queen_moves(board: &Board) -> Vec<Move> {}
//pub fn generate_king_moves(board: &Board) -> Vec<Move> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    #[test]
    fn generates_correct_pawn_moves() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        let moves = generate_pawn_moves(&board);
        assert_eq!(moves.len(), 16);
    }
    #[test]
    fn generates_correct_knight_moves() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        let moves = generate_knight_moves(&board);
        assert_eq!(moves.len(), 4);
    }
}
