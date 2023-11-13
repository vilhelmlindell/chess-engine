use super::attack_tables::*;
use crate::board::bitboard::Bitboard;
use crate::board::direction::Direction;
use crate::board::piece::{Piece, PieceType};
use crate::board::piece_move::Move;
use crate::board::piece_move::MoveType;
use crate::board::Board;
use crate::board::Side;
use crate::board::*;
use std::cmp::Ordering;

const WHITE_KINGSIDE_SQUARES: Bitboard = Bitboard(0x06000000000000000);
const WHITE_KINGSIDE_KING_SQUARES: [usize; 2] = [61, 62];
const WHITE_QUEENSIDE_SQUARES: Bitboard = Bitboard(0x0E00000000000000);
const WHITE_QUEENSIDE_KING_SQUARES: [usize; 2] = [59, 58];
const BLACK_KINGSIDE_SQUARES: Bitboard = Bitboard(0x0000000000000060);
const BLACK_KINGSIDE_KING_SQUARES: [usize; 2] = [5, 6];
const BLACK_QUEENSIDE_SQUARES: Bitboard = Bitboard(0x000000000000000E);
const BLACK_QUEENSIDE_KING_SQUARES: [usize; 2] = [3, 2];

pub fn generate_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::<Move>::with_capacity(250);

    generate_king_moves(board, &mut moves);

    let king_square = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();
    let king_attackers = board.attackers(king_square, board.side);
    let num_attackers = king_attackers.count_ones();

    match num_attackers.cmp(&1) {
        // only king moves valid if double check
        Ordering::Greater => return moves,
        // otherwise resolve the check
        Ordering::Equal => {
            resolve_single_check(board, king_attackers.lsb(), &mut moves);
            return moves;
        }
        _ => {}
    }

    generate_pawn_moves(board, &mut moves);
    generate_knight_moves(board, &mut moves);
    generate_bishop_moves(board, &mut moves);
    generate_rook_moves(board, &mut moves);
    generate_queen_moves(board, &mut moves);
    generate_castling_moves(board, &mut moves);

    moves
}

fn generate_pawn_moves(board: &Board, moves: &mut Vec<Move>) {
    let up_left: Direction = match board.side {
        Side::White => Direction::NorthWest,
        Side::Black => Direction::SouthEast,
    };
    let up_right: Direction = match board.side {
        Side::White => Direction::NorthEast,
        Side::Black => Direction::SouthWest,
    };
    let bitboard = board.piece_squares[Piece::new(PieceType::Pawn, board.side)];

    let mut pushed_pawns = push_pawns(bitboard, !board.occupied_squares, board.side);
    let rank = if board.side == Side::White { RANK_3 } else { RANK_6 };
    let double_pushed_pawns = push_pawns(pushed_pawns & rank, !board.occupied_squares, board.side);

    let mut promoted_pawns = pushed_pawns & (RANK_1 | RANK_8);
    pushed_pawns ^= promoted_pawns;

    add_moves_from_bitboard(board, pushed_pawns, |to| Move::new((to as i32 - Direction::up(board.side).value()) as usize, to, MoveType::Normal), moves);
    add_moves_from_bitboard(board,
        double_pushed_pawns,
        |to| Move::new((to as i32 - Direction::up(board.side).value() * 2) as usize, to, MoveType::DoublePush),
        moves,
    );

    for piece_type in PieceType::promotions() {
        add_moves_from_bitboard(board, 
            promoted_pawns,
            |to| Move::new((to as i32 - Direction::up(board.side).value()) as usize, to, MoveType::Promotion(piece_type)),
            moves,
        );
    }

    let mut capturing_pawns_up_left = bitboard.shift(up_left) & board.enemy_squares();
    promoted_pawns = capturing_pawns_up_left & (RANK_1 | RANK_8);
    capturing_pawns_up_left ^= promoted_pawns;

    add_moves_from_bitboard(board, capturing_pawns_up_left, |to| Move::new((to as i32 - up_left.value()) as usize, to, MoveType::Normal), moves);

    for piece_type in PieceType::promotions() {
        add_moves_from_bitboard(board, promoted_pawns, |to| Move::new((to as i32 - up_left.value()) as usize, to, MoveType::Promotion(piece_type)), moves);
    }
    let mut capturing_pawns_up_right = bitboard.shift(up_right) & board.enemy_squares();
    promoted_pawns = capturing_pawns_up_right & (RANK_1 | RANK_8);
    capturing_pawns_up_right ^= promoted_pawns;

    add_moves_from_bitboard(board, capturing_pawns_up_right, |to| Move::new((to as i32 - up_right.value()) as usize, to, MoveType::Normal), moves);

    for piece_type in PieceType::promotions() {
        add_moves_from_bitboard(board, promoted_pawns, |to| Move::new((to as i32 - up_right.value()) as usize, to, MoveType::Promotion(piece_type)), moves);
    }

    generate_en_passant_moves(board, moves);
}
fn generate_knight_moves(board: &Board, moves: &mut Vec<Move>) {
    let bitboard = board.piece_squares[Piece::new(PieceType::Knight, board.side)];

    for from in bitboard {
        let attack_bitboard = KNIGHT_ATTACK_MASKS[from] & !board.friendly_squares();
        add_moves_from_bitboard(board, attack_bitboard, |to| Move::new(from, to, MoveType::Normal), moves);
    }
}
fn generate_bishop_moves(board: &Board, moves: &mut Vec<Move>) {
    let bitboard = board.piece_squares[Piece::new(PieceType::Bishop, board.side)];

    for from in bitboard {
        let attack_bitboard = bishop_attacks(from, board.occupied_squares) & !board.friendly_squares();
        add_moves_from_bitboard(board, attack_bitboard, |to| Move::new(from, to, MoveType::Normal), moves);
    }
}
fn generate_rook_moves(board: &Board, moves: &mut Vec<Move>) {
    let bitboard = board.piece_squares[Piece::new(PieceType::Rook, board.side)];

    for from in bitboard {
        let attack_bitboard = rook_attacks(from, board.occupied_squares) & !board.friendly_squares();

        add_moves_from_bitboard(board, attack_bitboard, |to| Move::new(from, to, MoveType::Normal), moves);
    }
}
fn generate_queen_moves(board: &Board, moves: &mut Vec<Move>) {
    let bitboard = board.piece_squares[Piece::new(PieceType::Queen, board.side)];

    for from in bitboard {
        let attack_bitboard = queen_attacks(from, board.occupied_squares) & !board.friendly_squares();

        add_moves_from_bitboard(board, attack_bitboard, |to| Move::new(from, to, MoveType::Normal), moves);
    }
}
fn generate_king_moves(board: &Board, moves: &mut Vec<Move>) {
    let from = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();
    let enemy_king_square = board.piece_squares[Piece::new(PieceType::King, board.side.enemy())];
    let attack_bitboard = KING_ATTACK_MASKS[from] & !board.friendly_squares();

    for to in attack_bitboard {
        if !board.king_attacked(from, to) && (enemy_king_square & KING_ATTACK_MASKS[to] == 0) {
            moves.push(Move::new(from, to, MoveType::Normal));
        }
    }
}
fn generate_castling_moves(board: &Board, moves: &mut Vec<Move>) {
    match board.side {
        Side::White => {
            if can_castle(board, WHITE_KINGSIDE_SQUARES, WHITE_KINGSIDE_KING_SQUARES) && board.state().castling_rights[Side::White].kingside {
                moves.push(Move::new(60, 62, MoveType::Castle { kingside: true }));
            }
            if can_castle(board, WHITE_QUEENSIDE_SQUARES, WHITE_QUEENSIDE_KING_SQUARES) && board.state().castling_rights[Side::White].queenside {
                moves.push(Move::new(60, 58, MoveType::Castle { kingside: false }));
            }
        }
        Side::Black => {
            if can_castle(board, BLACK_KINGSIDE_SQUARES, BLACK_KINGSIDE_KING_SQUARES) && board.state().castling_rights[Side::Black].kingside {
                moves.push(Move::new(4, 6, MoveType::Castle { kingside: true }));
            }
            if can_castle(board, BLACK_QUEENSIDE_SQUARES, BLACK_QUEENSIDE_KING_SQUARES) && board.state().castling_rights[Side::Black].queenside {
                moves.push(Move::new(4, 2, MoveType::Castle { kingside: false }));
            }
        }
    };
}
fn generate_en_passant_moves(board: &Board, moves: &mut Vec<Move>) {
    if let Some(to) = board.state().en_passant_square {
        let mut en_passant_pawns = board.piece_squares[Piece::new(PieceType::Pawn, board.side)] & PAWN_ATTACKS[board.side][to];

        if en_passant_pawns == 0 {
            return;
        }

        let square = en_passant_pawns.pop_lsb();
        let square2 = en_passant_pawns.lsb();

        if square2 != 64 {
            let move1 = Move::new(square, to, MoveType::EnPassant);
            if legal(board, move1) {
                moves.push(move1);
            }
            let move2 = Move::new(square2, to, MoveType::EnPassant);
            if legal(board, move2) {
                moves.push(move2);
            }
            return;
        }

        let target_square = (to as i32 + Direction::down(board.side).value()) as usize;
        let rank = RANKS[7 - square / 8];
        let attackers = board.piece_squares[Piece::new(PieceType::Queen, board.side.enemy())] | board.piece_squares[Piece::new(PieceType::Rook, board.side.enemy())] & rank;

        if attackers == 0 {
            let mov = Move::new(square, to, MoveType::EnPassant);
            if legal(board, mov) {
                moves.push(mov);
            }
            return;
        }

        let king = board.piece_squares[Piece::new(PieceType::King, board.side)];

        if king & rank == 0 {
            let mov = Move::new(square, to, MoveType::EnPassant);
            if legal(board, mov) {
                moves.push(mov);
            }
            return;
        }

        let king_square = king.lsb();

        let attacker_square = if king_square < square { attackers.lsb() } else { attackers.msb() };
        let blockers = Bitboard::from_square(square) | Bitboard::from_square(target_square);

        // Check if performing the move will expose the king
        let expected_blockers = (BETWEEN_RAYS[king_square][attacker_square] ^ Bitboard::from_square(attacker_square)) & board.occupied_squares;

        if expected_blockers != blockers {
            let mov = Move::new(square, to, MoveType::EnPassant);
            if legal(board, mov) {
                moves.push(mov);
            }
        }
    }
}
fn can_castle(board: &Board, squares: Bitboard, king_squares: [usize; 2]) -> bool {
    // Check if all squares are unoccupied and not attacked
    let is_blocked = squares & board.occupied_squares != 0;
    let is_intercepted = king_squares.iter().any(|sq| board.attacked(*sq));
    !is_blocked && !is_intercepted
}
fn resolve_single_check(board: &Board, attacker_square: usize, moves: &mut Vec<Move>) {
    let king_square = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();

    // if the checker is a slider, we can block the check
    if board.squares[attacker_square].unwrap().piece_type().is_slider() {
        let mut attack_ray = BETWEEN_RAYS[king_square][attacker_square] ^ Bitboard::from_square(attacker_square);

        let pawns = board.piece_squares[Piece::new(PieceType::Pawn, board.side)];
        let mut pushed_pawns = push_pawns(pawns, !board.occupied_squares, board.side);
        let promoting_pawns = pushed_pawns & (RANK_1 | RANK_8);
        if promoting_pawns != 0 {
            pushed_pawns ^= promoting_pawns;
            for piece_type in PieceType::promotions() {
                add_moves_from_bitboard(board, 
                    promoting_pawns & attack_ray,
                    |to| Move::new((to as i32 + Direction::down(board.side).value()) as usize, to, MoveType::Promotion(piece_type)),
                    moves,
                );
            }
        }
        let rank = if board.side == Side::White { RANK_3 } else { RANK_6 };
        let double_pushed_pawns = push_pawns(pushed_pawns & rank, !board.occupied_squares, board.side);
        add_moves_from_bitboard(board, 
            pushed_pawns & attack_ray,
            |to| Move::new((to as i32 + Direction::down(board.side).value()) as usize, to, MoveType::Normal),
            moves,
        );

        add_moves_from_bitboard(board, 
            double_pushed_pawns & attack_ray,
            |to| Move::new((to as i32 + Direction::down(board.side).value() * 2) as usize, to, MoveType::DoublePush),
            moves,
        );

        // look for en passant blocks or captures
        if let Some(to) = board.state().en_passant_square {
            if attack_ray & Bitboard::from_square(to) != 0 || (to as i32 + Direction::down(board.side).value()) as usize == attacker_square {
                generate_en_passant_moves(board, moves);
            }
        }

        while attack_ray != 0 {
            let intercept_square = attack_ray.pop_lsb();
            let blockers = board.attackers(intercept_square, board.side.enemy()) & !board.piece_squares[Piece::new(PieceType::Pawn, board.side)];
            add_moves_from_bitboard(board, blockers, |from| Move::new(from, intercept_square, MoveType::Normal), moves);
        }
    }

    if let Some(to) = board.state().en_passant_square {
        if (to as i32 + Direction::down(board.side).value()) as usize == attacker_square {
            generate_en_passant_moves(board, moves);
        }
    }

    // try capturing the checker
    let mut capturers = board.attackers(attacker_square, board.side.enemy());
    let promoting_pawns = (capturers & board.piece_squares[Piece::new(PieceType::Pawn, board.side)]) & if board.side == Side::White { RANK_7 } else { RANK_2 };
    if promoting_pawns != 0 {
        capturers ^= promoting_pawns;
        for piece_type in PieceType::promotions() {
            add_moves_from_bitboard(board, promoting_pawns, |from| Move::new(from, attacker_square, MoveType::Promotion(piece_type)), moves);
        }
    }
    add_moves_from_bitboard(board, capturers, |from| Move::new(from, attacker_square, MoveType::Normal), moves);
}
fn add_moves_from_bitboard<F: Fn(usize) -> Move>(board: &Board, bitboard: Bitboard, mov: F, moves: &mut Vec<Move>) {
    for to in bitboard {
        let mov = mov(to);
        if legal(board, mov) {
            moves.push(mov);
        }
    }
}
fn legal(board: &Board, mov: Move) -> bool {
    // a non king move is only legal if the piece isn't pinned or it's moving along the ray
    // between the piece and the king
    board.absolute_pinned_squares.bit(mov.from) == 0 || Board::aligned(mov.to, mov.from, board.piece_squares[Piece::new(PieceType::King, board.side)].lsb())
}
fn push_pawns(pawns: Bitboard, empty_squares: Bitboard, side_to_move: Side) -> Bitboard {
    (pawns.north() << ((side_to_move.value()) << 4)) & empty_squares
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pawn_moves() {
        let board = Board::start_pos();
        let mut moves = Vec::<Move>::new();
        board.generate_pawn_moves(&mut moves);
        assert_eq!(moves.len(), 16);
        let board = Board::from_fen("8/8/3p1p2/3PpP2/8/1k6/2p5/Kn6 w - e6 0 1");
        let mut moves = board.generate_moves();
        moves.sort();
        let mut expected_moves = vec![Move::new(27, 20, MoveType::EnPassant), Move::new(29, 20, MoveType::EnPassant)];
        expected_moves.sort();
        assert_eq!(moves, expected_moves);
    }
    #[test]
    fn test_knight_moves() {
        let board = Board::start_pos();
        let mut moves = Vec::<Move>::new();
        board.generate_knight_moves(&mut moves);
        assert_eq!(moves.len(), 4);
    }
    #[test]
    fn test_move_legality() {
        let board = Board::from_fen("4k3/8/4q3/8/8/4R3/8/4K3 w - - 0 1");
        assert!(legal(&board, Move::new(44, 36, MoveType::Normal)));
        assert!(legal(&board, Move::new(44, 28, MoveType::Normal)));
        assert!(legal(&board, Move::new(44, 52, MoveType::Normal)));
        assert!(!legal(&board, Move::new(44, 43, MoveType::Normal)));
        assert!(!legal(&board, Move::new(44, 42, MoveType::Normal)));
        assert!(!legal(&board, Move::new(44, 45, MoveType::Normal)));
    }
    #[test]
    fn test_resolve_single_check() {
        let board = Board::from_fen("rnb1kbnr/ppppqppp/8/8/8/8/3P1P2/4KN2 w kq - 0 1");
        let mut expected_moves = vec![Move::new(61, 44, MoveType::Normal), Move::new(60, 59, MoveType::Normal)];
        expected_moves.sort();
        let mut moves = Vec::new();
        board.generate_king_moves(&mut moves);
        board.resolve_single_check(12, &mut moves);
        moves.sort();
        assert_eq!(moves, expected_moves);

        let board = Board::from_fen("rnb1kbnr/ppppqppp/8/8/1B6/8/3P1P2/3RKR2 w kq - 0 1");
        let mut expected_moves = vec![Move::new(33, 12, MoveType::Normal)];
        expected_moves.sort();
        let mut moves = Vec::new();
        board.generate_king_moves(&mut moves);
        board.resolve_single_check(12, &mut moves);
        moves.sort();
        assert_eq!(moves, expected_moves);

        let board = Board::from_fen("2k5/8/3b4/2PPpP2/2PKP3/2PPP3/8/8 w - e6 0 1");
        let mut expected_moves = vec![Move::new(27, 20, MoveType::EnPassant), Move::new(29, 20, MoveType::EnPassant)];
        expected_moves.sort();
        let mut moves = Vec::new();
        board.generate_king_moves(&mut moves);
        board.resolve_single_check(28, &mut moves);
        moves.sort();
        assert_eq!(moves, expected_moves);

        let board = Board::from_fen("6k1/8/1b6/8/2P5/8/3P2PP/5NKR w - - 0 1");
        let mut expected_moves = vec![Move::new(61, 44, MoveType::Normal), Move::new(34, 26, MoveType::Normal), Move::new(51, 35, MoveType::DoublePush)];
        expected_moves.sort();
        let mut moves = Vec::new();
        board.generate_king_moves(&mut moves);
        board.resolve_single_check(17, &mut moves);
        moves.sort();
        assert_eq!(moves, expected_moves);
    }
    #[test]
    fn test_castling() {
        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        let mut expected_moves = vec![Move::new(60, 62, MoveType::Castle { kingside: true }), Move::new(60, 58, MoveType::Castle { kingside: false })];
        expected_moves.sort();
        let mut moves = Vec::new();
        board.generate_castling_moves(&mut moves);
        moves.sort();
        assert_eq!(moves, expected_moves);
    }
}
