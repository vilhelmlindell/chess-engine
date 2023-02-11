use crate::bitboard::*;
use crate::board::Board;
use crate::board::CastlingRights;
use crate::board::Side;
use crate::direction::Direction;
use crate::piece::{Piece, PieceType};
use crate::piece_move::Move;
use crate::piece_move::MoveType;
use crate::tables::*;

pub const WHITE_KINGSIDE_CASTLING_MASK: Bitboard = Bitboard(0x6000000000000000);
pub const WHITE_QUEENSIDE_CASTLING_MASK: Bitboard = Bitboard(0x0E00000000000000);
pub const BLACK_KINGSIDE_CASTLING_MASK: Bitboard = Bitboard(0x0000000000000060);
pub const BLACK_QUEENSIDE_CASTLING_MASK: Bitboard = Bitboard(0x000000000000000E);

fn push_pawns(pawns: &Bitboard, empty_squares: &Bitboard, side_to_move: &Side) -> Bitboard {
    (pawns.north() >> ((side_to_move.value()) << 4)) & *empty_squares
}

impl Board {
    pub fn generate_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::<Move>::with_capacity(238);
        self.generate_pawn_moves(&mut moves);
        self.generate_knight_moves(&mut moves);
        self.generate_bishop_moves(&mut moves);
        self.generate_rook_moves(&mut moves);
        self.generate_queen_moves(&mut moves);
        self.generate_king_moves(&mut moves);
        moves
    }
    fn generate_pawn_moves(&mut self, moves: &mut Vec<Move>) {
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

        if let Some(end_square) = self.en_passant_square {
            let mut en_passant_pawns = bitboard & PAWN_ATTACKS[self.side_to_move.enemy()][end_square as usize];
            while en_passant_pawns != 0 {
                let square = en_passant_pawns.pop_lsb();
                moves.push(Move::new(square, end_square, MoveType::EnPassant));
            }
        }

        self.en_passant_square = None;

        let mut pushed_pawns = push_pawns(&bitboard, &!self.occupied_squares, &self.side_to_move);
        let mut double_pushed_pawns = push_pawns(&pushed_pawns, &!self.occupied_squares, &self.side_to_move);

        while pushed_pawns != 0 {
            let square = pushed_pawns.pop_lsb();
            moves.push(Move::new((square as i32 - up.value()) as u32, square, MoveType::Normal));
        }
        while double_pushed_pawns != 0 {
            let square = double_pushed_pawns.pop_lsb();
            self.en_passant_square = Some((square as i32 - up.value()) as u32);
            moves.push(Move::new((square as i32 - up.value() * 2) as u32, square, MoveType::Normal));
        }
        let mut capturing_pawns_up_left = bitboard.shift(up_left) & self.enemy_squares();
        let mut capturing_pawns_up_right = bitboard.shift(up_right) & self.enemy_squares();

        while capturing_pawns_up_left != 0 {
            let square = capturing_pawns_up_left.pop_lsb();
            moves.push(Move::new((square as i32 - up_left.value()) as u32, square, MoveType::Normal));
        }
        while capturing_pawns_up_right != 0 {
            let square = capturing_pawns_up_right.pop_lsb();
            moves.push(Move::new((square as i32 - up_right.value()) as u32, square, MoveType::Normal));
        }
    }
    fn generate_knight_moves(&self, moves: &mut Vec<Move>) {
        let mut bitboard = self.piece_bitboards[Piece::new(&PieceType::Knight, &self.side_to_move)];

        while bitboard != 0 {
            let square = bitboard.pop_lsb();
            let mut attack_bitboard = KNIGHT_ATTACK_MASKS[square as usize].clone() & !self.friendly_squares();
            while attack_bitboard != 0 {
                let end_square = attack_bitboard.pop_lsb();
                moves.push(Move::new(square, end_square, MoveType::Normal));
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
                moves.push(Move::new(square, end_square, MoveType::Normal));
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
                moves.push(Move::new(square, end_square, MoveType::Normal));
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
                moves.push(Move::new(square, end_square, MoveType::Normal));
            }
        }
    }
    fn generate_king_moves(&mut self, moves: &mut Vec<Move>) {
        let mut bitboard = self.piece_bitboards[Piece::new(&PieceType::King, &self.side_to_move)];

        while bitboard != 0 {
            let square = bitboard.pop_lsb();
            let mut attack_bitboard = KING_ATTACK_MASKS[square as usize].clone() & !self.friendly_squares();
            while attack_bitboard != 0 {
                let end_square = attack_bitboard.pop_lsb();
                moves.push(Move::new(square, end_square, MoveType::Normal));
            }
            let castling_rights = self.castlings_rights[self.side_to_move];
            match self.side_to_move {
                Side::White => {
                    if let CastlingRights::King | CastlingRights::All = castling_rights {
                        if WHITE_KINGSIDE_CASTLING_MASK & !self.occupied_squares != WHITE_KINGSIDE_CASTLING_MASK || self.is_attacked(60) || self.is_attacked(61) || self.is_attacked(62) {
                            self.castlings_rights[self.side_to_move] = CastlingRights::None;
                        } else {
                            moves.push(Move::new(60, 62, MoveType::Castle));
                        }
                    }
                    if let CastlingRights::Queen | CastlingRights::All = castling_rights {
                        if WHITE_QUEENSIDE_CASTLING_MASK & !self.occupied_squares != WHITE_QUEENSIDE_CASTLING_MASK || self.is_attacked(60) || self.is_attacked(59) || self.is_attacked(58) {
                            self.castlings_rights[self.side_to_move] = CastlingRights::None;
                        } else {
                            moves.push(Move::new(60, 58, MoveType::Castle));
                        }
                    }
                }
                Side::Black => {
                    if let CastlingRights::King | CastlingRights::All = castling_rights {
                        if BLACK_KINGSIDE_CASTLING_MASK & !self.occupied_squares != BLACK_KINGSIDE_CASTLING_MASK || self.is_attacked(4) || self.is_attacked(5) || self.is_attacked(6) {
                            self.castlings_rights[self.side_to_move] = CastlingRights::None;
                        } else {
                            moves.push(Move::new(4, 6, MoveType::Castle));
                        }
                    }
                    if let CastlingRights::Queen | CastlingRights::All = castling_rights {
                        if BLACK_QUEENSIDE_CASTLING_MASK & !self.occupied_squares != BLACK_QUEENSIDE_CASTLING_MASK || self.is_attacked(4) || self.is_attacked(3) || self.is_attacked(2) {
                            self.castlings_rights[self.side_to_move] = CastlingRights::None;
                        } else {
                            moves.push(Move::new(4, 2, MoveType::Castle));
                        }
                    }
                }
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
