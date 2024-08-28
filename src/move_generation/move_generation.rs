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

use arrayvec::ArrayVec;

pub const MAX_LEGAL_MOVES: usize = 218;

const WHITE_KINGSIDE_MASK: Bitboard = Bitboard(0x06000000000000000);
const WHITE_KINGSIDE_SQUARES: [usize; 2] = [61, 62];
const WHITE_QUEENSIDE_MASK: Bitboard = Bitboard(0x0E00000000000000);
const WHITE_QUEENSIDE_SQUARES: [usize; 2] = [59, 58];
const BLACK_KINGSIDE_MASK: Bitboard = Bitboard(0x0000000000000060);
const BLACK_KINGSIDE_SQUARES: [usize; 2] = [5, 6];
const BLACK_QUEENSIDE_MASK: Bitboard = Bitboard(0x000000000000000E);
const BLACK_QUEENSIDE_SQUARES: [usize; 2] = [3, 2];

pub fn generate_moves(board: &Board) -> ArrayVec<Move, MAX_LEGAL_MOVES> {
    let mut moves = ArrayVec::<Move, MAX_LEGAL_MOVES>::new();

    generate_king_moves(&mut moves, board);

    let king_square = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();
    let king_attackers = board.attackers(king_square, board.side);
    let num_attackers = king_attackers.count_ones();

    match num_attackers.cmp(&1) {
        // only king moves valid if double check
        Ordering::Greater => return moves,
        // otherwise resolve the check
        Ordering::Equal => {
            resolve_single_check(king_attackers.lsb(), &mut moves, board);
            return moves;
        }
        _ => {}
    }

    generate_pawn_moves(&mut moves, board);
    generate_knight_moves(&mut moves, board);
    generate_bishop_moves(&mut moves, board);
    generate_rook_moves(&mut moves, board);
    generate_queen_moves(&mut moves, board);
    generate_castling_moves(&mut moves, board);

    moves
}

#[inline(always)]
fn generate_pawn_moves(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
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

    if pushed_pawns != 0 {
        add_moves_from_bitboard(
            &|to| Move::new((to as i32 - Direction::up(board.side).value()) as usize, to, MoveType::Normal),
            moves,
            pushed_pawns,
            board,
        );
    }
    if double_pushed_pawns != 0 {
        add_moves_from_bitboard(
            &|to| Move::new((to as i32 - Direction::up(board.side).value() * 2) as usize, to, MoveType::DoublePush),
            moves,
            double_pushed_pawns,
            board,
        );
    }

    if promoted_pawns != 0 {
        for promotion_type in MoveType::PROMOTIONS {
            add_moves_from_bitboard(
                &|to| Move::new((to as i32 - Direction::up(board.side).value()) as usize, to, promotion_type),
                moves,
                promoted_pawns,
                board,
            );
        }
    }

    let mut capturing_pawns_up_left = bitboard.shift(up_left) & board.enemy_squares();
    promoted_pawns = capturing_pawns_up_left & (RANK_1 | RANK_8);
    capturing_pawns_up_left ^= promoted_pawns;

    if capturing_pawns_up_left != 0 {
        add_moves_from_bitboard(&|to| Move::new((to as i32 - up_left.value()) as usize, to, MoveType::Normal), moves, capturing_pawns_up_left, board);
    }

    if promoted_pawns != 0 {
        for promotion_type in MoveType::PROMOTIONS {
            add_moves_from_bitboard(&|to| Move::new((to as i32 - up_left.value()) as usize, to, promotion_type), moves, promoted_pawns, board);
        }
    }
    let mut capturing_pawns_up_right = bitboard.shift(up_right) & board.enemy_squares();
    promoted_pawns = capturing_pawns_up_right & (RANK_1 | RANK_8);
    capturing_pawns_up_right ^= promoted_pawns;

    add_moves_from_bitboard(&|to| Move::new((to as i32 - up_right.value()) as usize, to, MoveType::Normal), moves, capturing_pawns_up_right, board);

    if promoted_pawns != 0 {
        for promotion_type in MoveType::PROMOTIONS {
            add_moves_from_bitboard(&|to| Move::new((to as i32 - up_right.value()) as usize, to, promotion_type), moves, promoted_pawns, board);
        }
    }

    generate_en_passant_moves(moves, board);
}
#[inline(always)]
fn generate_knight_moves(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    let bitboard = board.piece_squares[Piece::new(PieceType::Knight, board.side)];

    for from in bitboard {
        let attack_bitboard = KNIGHT_ATTACK_MASKS[from] & !board.friendly_squares();
        add_moves_from_bitboard(&|to| Move::new(from, to, MoveType::Normal), moves, attack_bitboard, board);
    }
}
#[inline(always)]
fn generate_bishop_moves(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    let bitboard = board.piece_squares[Piece::new(PieceType::Bishop, board.side)];

    for from in bitboard {
        let attack_bitboard = bishop_attacks(from, board.occupied_squares) & !board.friendly_squares();
        add_moves_from_bitboard(&|to| Move::new(from, to, MoveType::Normal), moves, attack_bitboard, board);
    }
}
#[inline(always)]
fn generate_rook_moves(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    let bitboard = board.piece_squares[Piece::new(PieceType::Rook, board.side)];

    for from in bitboard {
        let attack_bitboard = rook_attacks(from, board.occupied_squares) & !board.friendly_squares();
        add_moves_from_bitboard(&|to| Move::new(from, to, MoveType::Normal), moves, attack_bitboard, board);
    }
}
#[inline(always)]
fn generate_queen_moves(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    let bitboard = board.piece_squares[Piece::new(PieceType::Queen, board.side)];

    for from in bitboard {
        let attack_bitboard = queen_attacks(from, board.occupied_squares) & !board.friendly_squares();
        add_moves_from_bitboard(&|to| Move::new(from, to, MoveType::Normal), moves, attack_bitboard, board);
    }
}
#[inline(always)]
fn generate_king_moves(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    let from = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();
    let enemy_king_square = board.piece_squares[Piece::new(PieceType::King, board.side.enemy())];
    let attack_bitboard = KING_ATTACK_MASKS[from] & !board.friendly_squares();

    for to in attack_bitboard {
        if !board.king_attacked(from, to) && (enemy_king_square & KING_ATTACK_MASKS[to] == 0) {
            moves.push(Move::new(from, to, MoveType::Normal));
        }
    }
}
#[inline(always)]
fn generate_castling_moves(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    match board.side {
        Side::White => {
            if can_castle(board, WHITE_KINGSIDE_MASK, WHITE_KINGSIDE_SQUARES) && board.state().castling_rights[Side::White].kingside {
                moves.push(Move::new(60, 62, MoveType::KingsideCastle));
            }
            if can_castle(board, WHITE_QUEENSIDE_MASK, WHITE_QUEENSIDE_SQUARES) && board.state().castling_rights[Side::White].queenside {
                moves.push(Move::new(60, 58, MoveType::QueensideCastle));
            }
        }
        Side::Black => {
            if can_castle(board, BLACK_KINGSIDE_MASK, BLACK_KINGSIDE_SQUARES) && board.state().castling_rights[Side::Black].kingside {
                moves.push(Move::new(4, 6, MoveType::KingsideCastle));
            }
            if can_castle(board, BLACK_QUEENSIDE_MASK, BLACK_QUEENSIDE_SQUARES) && board.state().castling_rights[Side::Black].queenside {
                moves.push(Move::new(4, 2, MoveType::QueensideCastle));
            }
        }
    };
}
#[inline(always)]
fn generate_en_passant_moves(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
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
#[inline(always)]
fn resolve_single_check(attacker_square: usize, moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    let king_square = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();

    // if the checker is a slider, we can block the check
    if board.squares[attacker_square].unwrap().piece_type().is_slider() {
        let mut attack_ray = BETWEEN_RAYS[king_square][attacker_square] ^ Bitboard::from_square(attacker_square);

        let pawns = board.piece_squares[Piece::new(PieceType::Pawn, board.side)];
        let mut pushed_pawns = push_pawns(pawns, !board.occupied_squares, board.side);
        let promoting_pawns = pushed_pawns & (RANK_1 | RANK_8);
        if promoting_pawns != 0 {
            pushed_pawns ^= promoting_pawns;
            for promotion_type in MoveType::PROMOTIONS {
                add_moves_from_bitboard(
                    &|to| Move::new((to as i32 + Direction::down(board.side).value()) as usize, to, promotion_type),
                    moves,
                    promoting_pawns & attack_ray,
                    board,
                );
            }
        }
        let rank = if board.side == Side::White { RANK_3 } else { RANK_6 };
        let double_pushed_pawns = push_pawns(pushed_pawns & rank, !board.occupied_squares, board.side);

        if pushed_pawns != 0 {
            add_moves_from_bitboard(
                &|to| Move::new((to as i32 + Direction::down(board.side).value()) as usize, to, MoveType::Normal),
                moves,
                pushed_pawns & attack_ray,
                board,
            );
        }

        if double_pushed_pawns != 0 {
            add_moves_from_bitboard(
                &|to| Move::new((to as i32 + Direction::down(board.side).value() * 2) as usize, to, MoveType::DoublePush),
                moves,
                double_pushed_pawns & attack_ray,
                board,
            );
        }

        // look for en passant blocks or captures
        if let Some(to) = board.state().en_passant_square {
            if attack_ray & Bitboard::from_square(to) != 0 || (to as i32 + Direction::down(board.side).value()) as usize == attacker_square {
                generate_en_passant_moves(moves, board);
            }
        }

        while attack_ray != 0 {
            let intercept_square = attack_ray.pop_lsb();
            let blockers = board.attackers(intercept_square, board.side.enemy()) & !board.piece_squares[Piece::new(PieceType::Pawn, board.side)];
            add_moves_from_bitboard(&|from| Move::new(from, intercept_square, MoveType::Normal), moves, blockers, board);
        }
    }

    if let Some(to) = board.state().en_passant_square {
        if (to as i32 + Direction::down(board.side).value()) as usize == attacker_square {
            generate_en_passant_moves(moves, board);
        }
    }

    // try capturing the checker
    let mut capturers = board.attackers(attacker_square, board.side.enemy());
    let promoting_pawns = (capturers & board.piece_squares[Piece::new(PieceType::Pawn, board.side)]) & if board.side == Side::White { RANK_7 } else { RANK_2 };
    if promoting_pawns != 0 {
        capturers ^= promoting_pawns;
        for promotion_type in MoveType::PROMOTIONS {
            add_moves_from_bitboard(&|from| Move::new(from, attacker_square, promotion_type), moves, promoting_pawns, board);
        }
    }
    add_moves_from_bitboard(&|from| Move::new(from, attacker_square, MoveType::Normal), moves, capturers, board);
}
#[inline(always)]
fn add_moves_from_bitboard<F: Fn(usize) -> Move>(mov: &F, moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, bitboard: Bitboard, board: &Board) {
    for to in bitboard {
        let mov = mov(to);
        if legal(board, mov) {
            moves.push(mov);
        }
    }
}
#[inline(always)]
fn legal(board: &Board, mov: Move) -> bool {
    // a non king move is only legal if the piece isn't pinned or it's moving along the ray
    // between the piece and the king
    board.absolute_pinned_squares.bit(mov.from()) == 0 || Board::aligned(mov.to(), mov.from(), board.piece_squares[Piece::new(PieceType::King, board.side)].lsb())
}
#[inline(always)]
fn push_pawns(pawns: Bitboard, empty_squares: Bitboard, side_to_move: Side) -> Bitboard {
    (pawns.north() << ((side_to_move.value()) << 4)) & empty_squares
}
#[inline(always)]
fn can_castle(board: &Board, squares: Bitboard, king_squares: [usize; 2]) -> bool {
    // Check if all squares are unoccupied and not attacked
    let is_blocked = squares & board.occupied_squares != 0;
    let is_intercepted = king_squares.iter().any(|sq| board.attacked(*sq));
    !is_blocked && !is_intercepted
}

#[cfg(test)]
mod tests {
    use super::*;

    //#[test]
    //fn test_pawn_moves() {
    //    let board = Board::start_pos();
    //    let mut moves = Vec::<Move>::new();
    //    generate_pawn_moves(&mut moves, &board);
    //    assert_eq!(moves.len(), 16);
    //    let board = Board::from_fen("8/8/3p1p2/3PpP2/8/1k6/2p5/Kn6 w - e6 0 1");
    //    let mut moves = board.generate_moves();
    //    moves.sort();
    //    let mut expected_moves = vec![Move::new(27, 20, MoveType::EnPassant), Move::new(29, 20, MoveType::EnPassant)];
    //    expected_moves.sort();
    //    assert_eq!(moves, expected_moves);
    //}
    //#[test]
    //fn test_knight_moves() {
    //    let board = Board::start_pos();
    //    let mut moves = Vec::<Move>::new();
    //    generate_knight_moves(&mut moves, &board);
    //    assert_eq!(moves.len(), 4);
    //}
    //#[test]
    //fn test_move_legality() {
    //    let board = Board::from_fen("4k3/8/4q3/8/8/4R3/8/4K3 w - - 0 1");
    //    assert!(legal(&board, Move::new(44, 36, MoveType::Normal)));
    //    assert!(legal(&board, Move::new(44, 28, MoveType::Normal)));
    //    assert!(legal(&board, Move::new(44, 52, MoveType::Normal)));
    //    assert!(!legal(&board, Move::new(44, 43, MoveType::Normal)));
    //    assert!(!legal(&board, Move::new(44, 42, MoveType::Normal)));
    //    assert!(!legal(&board, Move::new(44, 45, MoveType::Normal)));
    //}
    //#[test]
    //fn test_resolve_single_check() {
    //    let board = Board::from_fen("rnb1kbnr/ppppqppp/8/8/8/8/3P1P2/4KN2 w kq - 0 1");
    //    let mut expected_moves = vec![Move::new(61, 44, MoveType::Normal), Move::new(60, 59, MoveType::Normal)];
    //    expected_moves.sort();
    //    let mut moves = Vec::new();
    //    generate_king_moves(&mut moves, &board);
    //    resolve_single_check(12, &mut moves, &board);
    //    moves.sort();
    //    assert_eq!(moves, expected_moves);

    //    let board = Board::from_fen("rnb1kbnr/ppppqppp/8/8/1B6/8/3P1P2/3RKR2 w kq - 0 1");
    //    let mut expected_moves = vec![Move::new(33, 12, MoveType::Normal)];
    //    expected_moves.sort();
    //    let mut moves = Vec::new();
    //    generate_king_moves(&mut moves, &board);
    //    resolve_single_check(12, &mut moves, &board);
    //    moves.sort();
    //    assert_eq!(moves, expected_moves);

    //    let board = Board::from_fen("2k5/8/3b4/2PPpP2/2PKP3/2PPP3/8/8 w - e6 0 1");
    //    let mut expected_moves = vec![Move::new(27, 20, MoveType::EnPassant), Move::new(29, 20, MoveType::EnPassant)];
    //    expected_moves.sort();
    //    let mut moves = Vec::new();
    //    generate_king_moves(&mut moves, &board);
    //    resolve_single_check(28, &mut moves, &board);
    //    moves.sort();
    //    assert_eq!(moves, expected_moves);

    //    let board = Board::from_fen("6k1/8/1b6/8/2P5/8/3P2PP/5NKR w - - 0 1");
    //    let mut expected_moves = vec![Move::new(61, 44, MoveType::Normal), Move::new(34, 26, MoveType::Normal), Move::new(51, 35, MoveType::DoublePush)];
    //    expected_moves.sort();
    //    let mut moves = Vec::new();
    //    generate_king_moves(&mut moves, &board);
    //    resolve_single_check(17, &mut moves, &board);
    //    moves.sort();
    //    assert_eq!(moves, expected_moves);
    //}
    //#[test]
    //fn test_castling() {
    //    let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    //    let mut expected_moves = vec![Move::new(60, 62, MoveType::Castle { kingside: true }), Move::new(60, 58, MoveType::Castle { kingside: false })];
    //    expected_moves.sort();
    //    let mut moves = Vec::new();
    //    generate_castling_moves(&mut moves, &board);
    //    moves.sort();
    //    assert_eq!(moves, expected_moves);
    //}
}
