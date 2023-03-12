use crate::attack_tables::*;
use crate::bitboard::*;
use crate::board::Board;
use crate::board::Side;
use crate::direction::Direction;
use crate::piece::{Piece, PieceType};
use crate::piece_move::Move;
use crate::piece_move::MoveType;

fn push_pawns(pawns: &Bitboard, empty_squares: &Bitboard, side_to_move: &Side) -> Bitboard {
    (pawns.north() << ((side_to_move.value()) << 4)) & *empty_squares
}

impl Board {
    pub fn generate_moves(&self) -> Vec<Move> {
        let mut moves = Vec::<Move>::with_capacity(238);
        self.generate_knight_moves(&mut moves);
        if self.is_attacked(self.piece_squares[Piece::new(&PieceType::King, &self.side_to_move)].lsb()) {
            return moves;
        }
        self.generate_bishop_moves(&mut moves);
        self.generate_rook_moves(&mut moves);
        self.generate_queen_moves(&mut moves);
        self.generate_castling_moves(&mut moves);
        moves
    }
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
        let bitboard = self.piece_squares[Piece::new(&PieceType::Pawn, &self.side_to_move)];

        //if let Some(end_square) = self.state().en_passant_square {
        //    let mut en_passant_pawns = bitboard & PAWN_ATTACKS[self.side_to_move.enemy()][end_square] & !self.occupied_squares;
        //    while en_passant_pawns != 0 {
        //        let start_square = en_passant_pawns.pop_lsb();
        //        moves.push(Move::new(start_square, end_square, MoveType::EnPassant));
        //    }
        //}

        let mut pushed_pawns = push_pawns(&bitboard, &!self.occupied_squares, &self.side_to_move);
        let mut double_pushed_pawns = push_pawns(&pushed_pawns, &!self.occupied_squares, &self.side_to_move);

        while pushed_pawns != 0 {
            let end_square = pushed_pawns.pop_lsb();
            let start_square = (end_square as i32 - up.value()) as usize;
            moves.push(Move::new(start_square, end_square, MoveType::Normal));
        }
        while double_pushed_pawns != 0 {
            let end_square = double_pushed_pawns.pop_lsb();
            let start_square = (end_square as i32 - up.value() * 2) as usize;
            moves.push(Move::new(start_square, end_square, MoveType::DoublePush));
        }
        let mut capturing_pawns_up_left = bitboard.shift(up_left) & self.enemy_squares();
        let mut capturing_pawns_up_right = bitboard.shift(up_right) & self.enemy_squares();

        while capturing_pawns_up_left != 0 {
            let end_square = capturing_pawns_up_left.pop_lsb();
            let start_square = (end_square as i32 - up_left.value()) as usize;
            moves.push(Move::new(start_square, end_square, MoveType::Normal));
        }
        while capturing_pawns_up_right != 0 {
            let end_square = capturing_pawns_up_right.pop_lsb();
            let start_square = (end_square as i32 - up_right.value()) as usize;
            moves.push(Move::new(start_square, end_square, MoveType::Normal));
        }
    }
    fn generate_knight_moves(&self, moves: &mut Vec<Move>) {
        let mut bitboard = self.piece_squares[Piece::new(&PieceType::Knight, &self.side_to_move)];

        while bitboard != 0 {
            let start_square = bitboard.pop_lsb();
            let mut attack_bitboard = KNIGHT_ATTACK_MASKS[start_square] & !self.friendly_squares();
            while attack_bitboard != 0 {
                let end_square = attack_bitboard.pop_lsb();
                moves.push(Move::new(start_square, end_square, MoveType::Normal));
            }
        }
    }
    fn generate_bishop_moves(&self, moves: &mut Vec<Move>) {
        let mut bitboard = self.piece_squares[Piece::new(&PieceType::Bishop, &self.side_to_move)];

        while bitboard != 0 {
            let start_square = bitboard.pop_lsb();
            let mut attack_bitboard = get_bishop_attacks(&start_square, &self.occupied_squares) & !self.friendly_squares();
            while attack_bitboard != 0 {
                let end_square = attack_bitboard.pop_lsb();
                moves.push(Move::new(start_square, end_square, MoveType::Normal));
            }
        }
    }
    fn generate_rook_moves(&self, moves: &mut Vec<Move>) {
        let mut bitboard = self.piece_squares[Piece::new(&PieceType::Rook, &self.side_to_move)];

        while bitboard != 0 {
            let start_square = bitboard.pop_lsb();
            let mut attack_bitboard = get_rook_attacks(&start_square, &self.occupied_squares) & !self.friendly_squares();
            while attack_bitboard != 0 {
                let end_square = attack_bitboard.pop_lsb();
                moves.push(Move::new(start_square, end_square, MoveType::Normal));
            }
        }
    }
    fn generate_queen_moves(&self, moves: &mut Vec<Move>) {
        let mut bitboard = self.piece_squares[Piece::new(&PieceType::Queen, &self.side_to_move)];

        while bitboard != 0 {
            let start_square = bitboard.pop_lsb();
            let mut attack_bitboard = get_queen_attacks(&start_square, &self.occupied_squares) & !self.friendly_squares();
            while attack_bitboard != 0 {
                let end_square = attack_bitboard.pop_lsb();
                moves.push(Move::new(start_square, end_square, MoveType::Normal));
            }
        }
    }
    fn generate_king_moves(&self, moves: &mut Vec<Move>) {
        let start_square = self.piece_squares[Piece::new(&PieceType::King, &self.side_to_move)].lsb();
        let mut attack_bitboard = KING_ATTACK_MASKS[start_square] & !self.friendly_squares();
        while attack_bitboard != 0 {
            let end_square = attack_bitboard.pop_lsb();
            if !self.is_attacked(end_square) {
                moves.push(Move::new(start_square, end_square, MoveType::Normal));
            }
        }
    }
    fn generate_castling_moves(&self, moves: &mut Vec<Move>) {
        let start_square = self.piece_squares[Piece::new(&PieceType::King, &self.side_to_move)].lsb();
        match self.side_to_move {
            Side::White => {
                if self.state().castling_rights[self.side_to_move].kingside {
                    moves.push(Move::new(60, 62, MoveType::Castle { kingside: true }));
                }
                if self.state().castling_rights[self.side_to_move].queenside {
                    moves.push(Move::new(60, 58, MoveType::Castle { kingside: false }));
                }
            }
            Side::Black => {
                if self.state().castling_rights[self.side_to_move].kingside {
                    moves.push(Move::new(4, 6, MoveType::Castle { kingside: true }));
                }
                if self.state().castling_rights[self.side_to_move].queenside {
                    moves.push(Move::new(4, 2, MoveType::Castle { kingside: false }));
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_correct_pawn_moves() {
        let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        let mut moves = Vec::<Move>::new();
        board.generate_pawn_moves(&mut moves);
        assert_eq!(moves.len(), 16);
    }
    #[test]
    fn generates_correct_knight_moves() {
        let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        let mut moves = Vec::<Move>::new();
        board.generate_knight_moves(&mut moves);
        assert_eq!(moves.len(), 4);
    }
}
