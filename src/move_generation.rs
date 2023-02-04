use crate::bitboard::*;
use crate::board::Side;
use crate::direction::Direction;
use crate::piece::{Piece, PieceType};
use crate::r#move::Move;
use crate::tables::*;

fn push_pawns(pawns: &Bitboard, empty_squares: &Bitboard, side_to_move: &Side) -> Bitboard {
    (pawns.north() >> ((side_to_move.value()) << 4)) & *empty_squares
}

impl Board {
    fn generate_pawn_moves(&self, moves: &mut Vec<Move>) {
        let up: &Direction = match self.side_to_move {
            Side::White => &Direction::North,
            Side::Black => &Direction::South,
        };
        let up_left: &Direction = match self.side_to_move {
            Side::White => &Direction::NorthWest,
            Side::Black => &Direction::SouthEast,
        };
        let up_right: &Direction = match self.side_to_move {
            Side::White => &Direction::NorthEast,
            Side::Black => &Direction::SouthWest,
        };
        let bitboard = self.piece_bitboards[Piece::new(&PieceType::Pawn, &self.side_to_move)];

        let mut pushed_pawns = push_pawns(&bitboard, &!self.occupied_squares, &self.side_to_move);
        let mut double_pushed_pawns = push_pawns(&pushed_pawns, &!self.occupied_squares, &self.side_to_move);

        while pushed_pawns != 0 {
            let square = pushed_pawns.pop_lsb();
            moves.push(Move::new((square as i32 - up.value()) as u32, square));
        }
        while double_pushed_pawns != 0 {
            let square = double_pushed_pawns.pop_lsb();
            moves.push(Move::new((square as i32 - up.value() * 2) as u32, square));
        }
        let mut capturing_pawns_up_left = bitboard.shift(up_left) & self.enemy_squares();
        let mut capturing_pawns_up_right = bitboard.shift(up_right) & self.enemy_squares();

        while capturing_pawns_up_left != 0 {
            let square = capturing_pawns_up_left.pop_lsb();
            moves.push(Move::new((square as i32 - up_left.value()) as u32, square));
        }
        while capturing_pawns_up_right != 0 {
            let square = capturing_pawns_up_right.pop_lsb();
            moves.push(Move::new((square as i32 - up_right.value()) as u32, square));
        }
    }
    fn generate_knight_moves(&self, moves: &mut Vec<Move>) {
        let mut bitboard = self.piece_bitboards[Piece::new(&PieceType::Knight, &self.side_to_move)];

        while bitboard != 0 {
            let square = bitboard.pop_lsb();
            let mut attack_bitboard = KNIGHT_ATTACK_MASKS[square as usize].clone() & !self.friendly_squares();
            while attack_bitboard != 0 {
                let end_square = attack_bitboard.pop_lsb();
                moves.push(Move::new(square, end_square));
            }
        }
    }
    fn generate_bishop_moves(&self, moves: &mut Vec<Move>) {
        let mut bitboard = self.piece_bitboards[Piece::new(&PieceType::Bishop, &self.side_to_move)];

        while bitboard != 0 {
            let square = bitboard.pop_lsb();
            let mut attack_bitboard = get_bishop_attacks(&(square as usize), &self.occupied_squares) & !self.friendly_squares();
            while attack_bitboard != 0 {
                let end_square = attack_bitboard.pop_lsb();
                moves.push(Move::new(square, end_square));
            }
        }
    }
    fn generate_rook_moves(&self, moves: &mut Vec<Move>) {
        let mut bitboard = self.piece_bitboards[Piece::new(&PieceType::Rook, &self.side_to_move)];

        while bitboard != 0 {
            let square = bitboard.pop_lsb();
            let mut attack_bitboard = get_rook_attacks(&(square as usize), &self.occupied_squares) & !self.friendly_squares();
            while attack_bitboard != 0 {
                let end_square = attack_bitboard.pop_lsb();
                moves.push(Move::new(square, end_square));
            }
        }
    }
    fn generate_queen_moves(&self, moves: &mut Vec<Move>) {
        let mut bitboard = self.piece_bitboards[Piece::new(&PieceType::Queen, &self.side_to_move)];

        while bitboard != 0 {
            let square = bitboard.pop_lsb();
            let mut attack_bitboard = get_queen_attacks(&(square as usize), &self.occupied_squares) & !self.friendly_squares();
            while attack_bitboard != 0 {
                let end_square = attack_bitboard.pop_lsb();
                moves.push(Move::new(square, end_square));
            }
        }
    }
    fn generate_king_moves(&self, moves: &mut Vec<Move>) {
        let mut bitboard = self.piece_bitboards[Piece::new(&PieceType::King, &self.side_to_move)];

        while bitboard != 0 {
            let square = bitboard.pop_lsb();
            let mut attack_bitboard = KING_ATTACK_MASKS[square as usize].clone() & !self.friendly_squares();
            while attack_bitboard != 0 {
                let end_square = attack_bitboard.pop_lsb();
                moves.push(Move::new(square, end_square));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Board;

    #[test]
    fn generates_correct_pawn_moves() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        let mut moves = Vec::<Move>::new();
        board.generate_pawn_moves(&mut moves);
        assert_eq!(moves.len(), 16);
    }
    #[test]
    fn generates_correct_knight_moves() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        let mut moves = Vec::<Move>::new();
        board.generate_knight_moves(&mut moves);
        assert_eq!(moves.len(), 4);
    }
}
