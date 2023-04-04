use crate::attack_tables::*;
use crate::bitboard::*;
use crate::board::Board;
use crate::board::Side;
use crate::direction::Direction;
use crate::piece::{Piece, PieceType};
use crate::piece_move::Move;
use crate::piece_move::MoveType;

const RANK_4: Bitboard = Bitboard(0x000000FF00000000);
const RANK_6: Bitboard = Bitboard(0x00000000FF000000);

fn push_pawns(pawns: &Bitboard, empty_squares: &Bitboard, side_to_move: &Side) -> Bitboard {
    (pawns.north() << ((side_to_move.value()) << 4)) & *empty_squares
}

impl Board {
    pub fn generate_moves(&self) -> Vec<Move> {
        let mut moves = Vec::<Move>::new();

        self.generate_king_moves(&mut moves);

        let king_square = self.piece_squares[Piece::new(&PieceType::King, &self.side_to_move)].lsb();
        let mut king_attackers = self.attackers(&king_square, &self.side_to_move);
        let num_attackers = king_attackers.count_ones();

        if num_attackers > 1 {
            return moves; // only king moves valid if double check
        }
        if num_attackers == 1 {
            let checker_square = king_attackers.pop_lsb();

            // if the checker is a slider, we can block the check
            if self.squares[checker_square].unwrap().piece_type().is_slider() {
                let mut attack_ray = BETWEEN_RAYS[king_square][checker_square];

                let pawns = self.piece_squares[Piece::new(&PieceType::Pawn, &self.side_to_move)];
                let mut pawn_blockers = attack_ray.shift(&Direction::down(&self.side_to_move)) & pawns;
                while pawn_blockers != 0 {
                    let to = pawn_blockers.pop_lsb();
                    let from = (to as i32 + Direction::up(&self.side_to_move).value()) as usize;
                    let mov = Move::new(from, to, MoveType::Normal);
                    if self.legal(&mov) {
                        moves.push(mov);
                    }
                }

                let double_push_squares = attack_ray & if self.side_to_move == Side::White { RANK_4 } else { RANK_6 };
                pawn_blockers = double_push_squares & pawns;
                while pawn_blockers != 0 {
                    let blocker_square = pawn_blockers.pop_lsb();
                    let mov = Move::new(
                        blocker_square,
                        (blocker_square as i32 + Direction::up(&self.side_to_move).value() * 2) as usize,
                        MoveType::Normal,
                    );
                    if self.legal(&mov) {
                        moves.push(mov);
                    }
                }

                while attack_ray != 0 {
                    let intercept_square = attack_ray.pop_lsb();
                    let blockers = self.attackers(&intercept_square, &self.side_to_move.enemy());
                    self.add_moves_from_bitboard(blockers, |from| Move::new(from, intercept_square, MoveType::Normal), &mut moves);

                    // look for en passant blocks
                    if let Some(to) = self.state().en_passant_square {
                        if attack_ray & Bitboard::from_square(&to) != 0 {
                            let en_passant_pawns = PAWN_ATTACKS[self.side_to_move.enemy()][to]
                                & self.piece_squares[Piece::new(&PieceType::Pawn, &self.side_to_move)];
                            self.add_moves_from_bitboard(en_passant_pawns, |from| Move::new(from, to, MoveType::Normal), &mut moves);
                        }
                    }
                }
            }
            // otherwise we can capture the checker
            while king_attackers != 0 {
                let attacker_square = king_attackers.pop_lsb();
                let mov = Move::new(attacker_square, checker_square, MoveType::Normal);
                if self.legal(&mov) {
                    moves.push(mov);
                }
            }
            return moves;
        }

        self.generate_queen_moves(&mut moves);
        self.generate_rook_moves(&mut moves);
        self.generate_bishop_moves(&mut moves);
        self.generate_knight_moves(&mut moves);
        self.generate_pawn_moves(&mut moves);
        self.generate_castling_moves(&mut moves);
        moves
    }
    fn resolve_single_check(moves: &mut Vec<Move>) {

    }
    fn legal(&self, mov: &Move) -> bool {
        // a non king move is only legal if the piece isn't pinned or it's moving along the ray
        // between the piece and the king
        self.absolute_pinned_squares.bit(&mov.from) == 0
            || Self::aligned(
            &mov.from,
            &mov.to,
            &self.piece_squares[Piece::new(&PieceType::Rook, &self.side_to_move)].lsb(),
            )
    }

    fn add_moves_from_bitboard<F: Fn(usize) -> Move>(&self, mut bitboard: Bitboard, mov: F, moves: &mut Vec<Move>) {
        while bitboard != 0 {
            let end_square = bitboard.pop_lsb();
            let mov = mov(end_square);
            if self.legal(&mov) {
                moves.push(mov);
            }
        }
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

        if let Some(to) = self.state().en_passant_square {
            let mut en_passant_pawns = bitboard & PAWN_ATTACKS[self.side_to_move.enemy()][to] & !self.occupied_squares;
            while en_passant_pawns != 0 {
                let from = en_passant_pawns.pop_lsb();
                moves.push(Move::new(from, to, MoveType::EnPassant));
            }
        }

        let pushed_pawns = push_pawns(&bitboard, &!self.occupied_squares, &self.side_to_move);
        let double_pushed_pawns = push_pawns(&pushed_pawns, &!self.occupied_squares, &self.side_to_move);

        self.add_moves_from_bitboard(pushed_pawns, |to| Move::new((to as i32 - up.value()) as usize, to, MoveType::Normal), moves);
        self.add_moves_from_bitboard(double_pushed_pawns, |to| Move::new((to as i32 - up.value() * 2) as usize, to, MoveType::DoublePush), moves);

        let capturing_pawns_up_left = bitboard.shift(up_left) & self.enemy_squares();
        let capturing_pawns_up_right = bitboard.shift(up_right) & self.enemy_squares();

        self.add_moves_from_bitboard(capturing_pawns_up_left, |to| Move::new((to as i32 - up_left.value()) as usize, to, MoveType::Normal), moves);
        self.add_moves_from_bitboard(capturing_pawns_up_right, |to| Move::new((to as i32 - up_right.value()) as usize, to, MoveType::Normal), moves);
    }
    fn generate_knight_moves(&self, moves: &mut Vec<Move>) {
        let mut bitboard = self.piece_squares[Piece::new(&PieceType::Knight, &self.side_to_move)];

        while bitboard != 0 {
            let from = bitboard.pop_lsb();
            let attack_bitboard = KNIGHT_ATTACK_MASKS[from] & !self.friendly_squares();
            self.add_moves_from_bitboard(attack_bitboard, |to| Move::new(from, to, MoveType::Normal), moves);
        }
    }
    fn generate_bishop_moves(&self, moves: &mut Vec<Move>) {
        let mut bitboard = self.piece_squares[Piece::new(&PieceType::Bishop, &self.side_to_move)];

        while bitboard != 0 {
            let from = bitboard.pop_lsb();
            let attack_bitboard = bishop_attacks(&from, &self.occupied_squares) & !self.friendly_squares();
            self.add_moves_from_bitboard(attack_bitboard, |to| Move::new(from, to, MoveType::Normal), moves);
        }
    }
    fn generate_rook_moves(&self, moves: &mut Vec<Move>) {
        let mut bitboard = self.piece_squares[Piece::new(&PieceType::Rook, &self.side_to_move)];

        while bitboard != 0 {
            let from = bitboard.pop_lsb();
            let attack_bitboard = rook_attacks(&from, &self.occupied_squares) & !self.friendly_squares();

            self.add_moves_from_bitboard(attack_bitboard, |to| Move::new(from, to, MoveType::Normal), moves);
        }
    }
    fn generate_queen_moves(&self, moves: &mut Vec<Move>) {
        let mut bitboard = self.piece_squares[Piece::new(&PieceType::Queen, &self.side_to_move)];

        while bitboard != 0 {
            let from = bitboard.pop_lsb();
            let attack_bitboard = queen_attacks(&from, &self.occupied_squares) & !self.friendly_squares();

            self.add_moves_from_bitboard(attack_bitboard, |to| Move::new(from, to, MoveType::Normal), moves);
        }
    }
    fn generate_king_moves(&self, moves: &mut Vec<Move>) {
        let from = self.piece_squares[Piece::new(&PieceType::King, &self.side_to_move)].lsb();
        let mut attack_bitboard = KING_ATTACK_MASKS[from] & !self.friendly_squares();
        while attack_bitboard != 0 {
            let to = attack_bitboard.pop_lsb();
            if !self.attacked(&to) {
                moves.push(Move::new(from, to, MoveType::Normal));
            }
        }
    }
    fn generate_castling_moves(&self, moves: &mut Vec<Move>) {
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
    fn test_pawn_moves() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        let mut moves = Vec::<Move>::new();
        board.generate_pawn_moves(&mut moves);
        assert_eq!(moves.len(), 16);
    }
    #[test]
    fn test_knight_moves() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        let mut moves = Vec::<Move>::new();
        board.generate_knight_moves(&mut moves);
        assert_eq!(moves.len(), 4);
    }
}
