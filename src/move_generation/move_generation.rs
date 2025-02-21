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
            if board.side == Side::White {
                resolve_single_check::<true>(king_attackers.lsb(), &mut moves, board);
                return moves;
            } else {
                resolve_single_check::<false>(king_attackers.lsb(), &mut moves, board);
                return moves;
            }
        }
        _ => {}
    }

    if board.side == Side::White {
        generate_pawn_moves::<true>(&mut moves, board);
        generate_castling_moves::<true>(&mut moves, board);
    } else {
        generate_pawn_moves::<false>(&mut moves, board);
        generate_castling_moves::<false>(&mut moves, board);
    }

    generate_knight_moves(&mut moves, board);
    generate_bishop_moves(&mut moves, board);
    generate_rook_moves(&mut moves, board);
    generate_queen_moves(&mut moves, board);

    moves
}
#[inline(never)]
fn generate_pawn_moves<const IS_WHITE: bool>(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    let (up_left, up_right, up) = if IS_WHITE { (Direction::NorthWest, Direction::NorthEast, Direction::North) } else { (Direction::SouthEast, Direction::SouthWest, Direction::South) };

    let (double_push_rank, pre_promotion_rank) = if IS_WHITE { (RANK_3, RANK_7) } else { (RANK_6, RANK_2) };

    let pawns = board.piece_squares[Piece::new(PieceType::Pawn, board.side)] & !pre_promotion_rank;
    let pushed = pawns.shift(up) & !board.occupied_squares;
    let double_pushed = (pushed & double_push_rank).shift(up) & !board.occupied_squares;

    let normal_move = |to, direction: Direction| Move::new((to as i32 - direction.value()) as usize, to, MoveType::Normal);
    let double_push_move = |to| Move::new((to as i32 - up.value() * 2) as usize, to, MoveType::DoublePush);

    add_moves(|to| normal_move(to, up), moves, pushed, board);
    add_moves(double_push_move, moves, double_pushed, board);

    let captures_up_right = pawns.shift(up_right) & board.enemy_squares();
    let captures_up_left = pawns.shift(up_left) & board.enemy_squares();

    add_moves(|to| normal_move(to, up_right), moves, captures_up_right, board);
    add_moves(|to| normal_move(to, up_left), moves, captures_up_left, board);

    let promotable = board.piece_squares[Piece::new(PieceType::Pawn, board.side)] & pre_promotion_rank;

    if promotable != 0 {
        let promotions_up = promotable.shift(up) & !board.occupied_squares;
        let promotions_up_right = promotable.shift(up_right) & board.enemy_squares();
        let promotions_up_left = promotable.shift(up_left) & board.enemy_squares();

        let promotion_move = |to, direction: Direction| (to as i32 - direction.value()) as usize;

        add_promotions(|to| promotion_move(to, up), moves, promotions_up, board);
        add_promotions(|to| promotion_move(to, up_right), moves, promotions_up_right, board);
        add_promotions(|to| promotion_move(to, up_left), moves, promotions_up_left, board);
    }

    generate_en_passant_moves(moves, board);
}
#[inline(never)]
fn generate_knight_moves(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    let bitboard = board.piece_squares[Piece::new(PieceType::Knight, board.side)];

    for from in bitboard {
        let attack_bitboard = get_knight_attack_mask(from) & !board.friendly_squares();
        add_moves(|to| Move::new(from, to, MoveType::Normal), moves, attack_bitboard, board);
    }
}
#[inline(never)]
fn generate_bishop_moves(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    let bitboard = board.piece_squares[Piece::new(PieceType::Bishop, board.side)];

    for from in bitboard {
        let attack_bitboard = bishop_attacks(from, board.occupied_squares) & !board.friendly_squares();
        add_moves(|to| Move::new(from, to, MoveType::Normal), moves, attack_bitboard, board);
    }
}
#[inline(never)]
fn generate_rook_moves(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    let bitboard = board.piece_squares[Piece::new(PieceType::Rook, board.side)];

    for from in bitboard {
        let attack_bitboard = rook_attacks(from, board.occupied_squares) & !board.friendly_squares();
        add_moves(|to| Move::new(from, to, MoveType::Normal), moves, attack_bitboard, board);
    }
}
#[inline(never)]
fn generate_queen_moves(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    let bitboard = board.piece_squares[Piece::new(PieceType::Queen, board.side)];

    for from in bitboard {
        let attack_bitboard = queen_attacks(from, board.occupied_squares) & !board.friendly_squares();
        add_moves(|to| Move::new(from, to, MoveType::Normal), moves, attack_bitboard, board);
    }
}
#[inline(never)]
fn generate_king_moves(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    let from = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();
    let enemy_king_square = board.piece_squares[Piece::new(PieceType::King, board.side.enemy())];
    let attack_bitboard = get_king_attack_mask(from) & !board.friendly_squares();

    for to in attack_bitboard {
        if !board.king_attacked(from, to) && (enemy_king_square & get_king_attack_mask(to) == 0) {
            unsafe { moves.push_unchecked(Move::new(from, to, MoveType::Normal)) };
        }
    }
}
#[inline(never)]
fn generate_castling_moves<const IS_WHITE: bool>(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    if IS_WHITE {
        if can_castle(board, WHITE_KINGSIDE_MASK, WHITE_KINGSIDE_SQUARES) && board.state().castling_rights[Side::White].kingside {
            unsafe { moves.push_unchecked(Move::new(60, 62, MoveType::KingsideCastle)) }
        }
        if can_castle(board, WHITE_QUEENSIDE_MASK, WHITE_QUEENSIDE_SQUARES) && board.state().castling_rights[Side::White].queenside {
            unsafe { moves.push_unchecked(Move::new(60, 58, MoveType::QueensideCastle)) }
        }
    } else {
        if can_castle(board, BLACK_KINGSIDE_MASK, BLACK_KINGSIDE_SQUARES) && board.state().castling_rights[Side::Black].kingside {
            unsafe { moves.push_unchecked(Move::new(4, 6, MoveType::KingsideCastle)) }
        }
        if can_castle(board, BLACK_QUEENSIDE_MASK, BLACK_QUEENSIDE_SQUARES) && board.state().castling_rights[Side::Black].queenside {
            unsafe { moves.push_unchecked(Move::new(4, 2, MoveType::QueensideCastle)) }
        }
    }
}
#[inline(never)]
fn generate_en_passant_moves(moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    if let Some(to) = board.state().en_passant_square {
        let mut en_passant_pawns = board.piece_squares[Piece::new(PieceType::Pawn, board.side)] & get_pawn_attack(board.side, to);

        if en_passant_pawns == 0 {
            return;
        }

        let square = en_passant_pawns.pop_lsb();
        let square2 = en_passant_pawns.lsb();

        if square2 != 64 {
            let move1 = Move::new(square, to, MoveType::EnPassant);
            if legal(board, move1.from(), move1.to()) {
                unsafe { moves.push_unchecked(move1) }
            }
            let move2 = Move::new(square2, to, MoveType::EnPassant);
            if legal(board, move2.from(), move2.to()) {
                unsafe { moves.push_unchecked(move2) }
            }
            return;
        }

        let target_square = (to as i32 + Direction::down(board.side).value()) as usize;
        let rank = RANKS[7 - square / 8];
        let attackers = board.piece_squares[Piece::new(PieceType::Queen, board.side.enemy())] | board.piece_squares[Piece::new(PieceType::Rook, board.side.enemy())] & rank;

        if attackers == 0 {
            let mov = Move::new(square, to, MoveType::EnPassant);
            if legal(board, mov.from(), mov.to()) {
                unsafe { moves.push_unchecked(mov) }
            }
            return;
        }

        let king = board.piece_squares[Piece::new(PieceType::King, board.side)];

        if king & rank == 0 {
            let mov = Move::new(square, to, MoveType::EnPassant);
            if legal(board, mov.from(), mov.to()) {
                unsafe { moves.push_unchecked(mov) }
            }
            return;
        }

        let king_square = king.lsb();

        let attacker_square = if king_square < square { attackers.lsb() } else { attackers.msb() };
        let blockers = Bitboard::from_square(square) | Bitboard::from_square(target_square);

        // Check if performing the move will expose the king
        let expected_blockers = (get_between_ray(king_square, attacker_square) ^ Bitboard::from_square(attacker_square)) & board.occupied_squares;

        if expected_blockers != blockers {
            let mov = Move::new(square, to, MoveType::EnPassant);
            if legal(board, mov.from(), mov.to()) {
                unsafe { moves.push_unchecked(mov) }
            }
        }
    }
}
#[inline(never)]
fn resolve_single_check<const IS_WHITE: bool>(attacker_square: usize, moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, board: &Board) {
    let king_square = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();

    // if the checker is a slider, we can block the check
    if board.squares[attacker_square].unwrap().piece_type().is_slider() {
        let mut attack_ray = get_between_ray(king_square, attacker_square) ^ Bitboard::from_square(attacker_square);

        let pawns = board.piece_squares[Piece::new(PieceType::Pawn, board.side)];
        let mut pushed_pawns = push_pawns::<IS_WHITE>(pawns, !board.occupied_squares);
        let promoting_pawns = pushed_pawns & (RANK_1 | RANK_8);
        if promoting_pawns != 0 {
            pushed_pawns ^= promoting_pawns;
            for promotion_type in MoveType::PROMOTIONS {
                add_moves(&|to| Move::new((to as i32 + Direction::down(board.side).value()) as usize, to, promotion_type), moves, promoting_pawns & attack_ray, board);
            }
        }
        let rank = if IS_WHITE { RANK_3 } else { RANK_6 };
        let double_pushed_pawns = push_pawns::<IS_WHITE>(pushed_pawns & rank, !board.occupied_squares);

        add_moves(|to| Move::new((to as i32 + Direction::down(board.side).value()) as usize, to, MoveType::Normal), moves, pushed_pawns & attack_ray, board);
        add_moves(|to| Move::new((to as i32 + Direction::down(board.side).value() * 2) as usize, to, MoveType::DoublePush), moves, double_pushed_pawns & attack_ray, board);

        // look for en passant blocks or captures
        if let Some(to) = board.state().en_passant_square {
            if attack_ray & Bitboard::from_square(to) != 0 || (to as i32 + Direction::down(board.side).value()) as usize == attacker_square {
                generate_en_passant_moves(moves, board);
            }
        }

        while attack_ray != 0 {
            let intercept_square = attack_ray.pop_lsb();
            let blockers = board.attackers(intercept_square, board.side.enemy()) & !board.piece_squares[Piece::new(PieceType::Pawn, board.side)];
            add_moves(|from| Move::new(from, intercept_square, MoveType::Normal), moves, blockers, board);
        }
    }

    if let Some(to) = board.state().en_passant_square {
        if (to as i32 + Direction::down(board.side).value()) as usize == attacker_square {
            generate_en_passant_moves(moves, board);
        }
    }

    // try capturing the checker
    let mut capturers = board.attackers(attacker_square, board.side.enemy());
    let promoting_pawns = (capturers & board.piece_squares[Piece::new(PieceType::Pawn, board.side)]) & if IS_WHITE { RANK_7 } else { RANK_2 };

    capturers ^= promoting_pawns;
    for promotion_type in MoveType::PROMOTIONS {
        add_moves(|from| Move::new(from, attacker_square, promotion_type), moves, promoting_pawns, board);
    }

    add_moves(|from| Move::new(from, attacker_square, MoveType::Normal), moves, capturers, board);
}
#[inline(always)]
fn add_moves<F: Fn(usize) -> Move>(mov: F, moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, bitboard: Bitboard, board: &Board) {
    for to in bitboard {
        let mov = mov(to);
        if legal(board, mov.from(), mov.to()) {
            unsafe { moves.push_unchecked(mov) }
        }
    }
}
#[inline(always)]
fn add_promotions<F: Fn(usize) -> usize>(from: F, moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, bitboard: Bitboard, board: &Board) {
    for to in bitboard {
        let from = from(to);
        if legal(board, from, to) {
            for promotion_type in MoveType::PROMOTIONS {
                let mov = Move::new(from, to, promotion_type);
                unsafe { moves.push_unchecked(mov) }
            }
        }
    }
}
#[inline(never)]
fn legal(board: &Board, from: usize, to: usize) -> bool {
    // a non king move is only legal if the piece isn't pinned or it's moving along the ray
    // between the piece and the king
    board.absolute_pinned_squares.bit(from) == 0 || Board::aligned(to, from, board.piece_squares[Piece::new(PieceType::King, board.side)].lsb())
}
#[inline(never)]
fn push_pawns<const IS_WHITE: bool>(pawns: Bitboard, empty_squares: Bitboard) -> Bitboard {
    if IS_WHITE {
        pawns.north() & empty_squares
    } else {
        pawns.south() & empty_squares
    }
}
#[inline(never)]
fn can_castle(board: &Board, squares: Bitboard, king_squares: [usize; 2]) -> bool {
    // Check if all squares are unoccupied and not attacked
    let is_blocked = squares & board.occupied_squares != 0;
    //let is_intercepted = king_squares.iter().any(|sq| board.attacked(*sq));
    let is_intercepted = board.attacked(king_squares[0]) || board.attacked(king_squares[1]);
    !is_blocked && !is_intercepted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pawn_moves() {
        let board = Board::start_pos();
        let mut moves = ArrayVec::<Move, MAX_LEGAL_MOVES>::new();
        generate_pawn_moves::<true>(&mut moves, &board);
        assert_eq!(moves.len(), 16);
        let board = Board::from_fen("8/8/3p1p2/3PpP2/8/1k6/2p5/Kn6 w - e6 0 1");
        let mut moves = generate_moves(&board);
        moves.sort();
        let mut expected_moves = ArrayVec::<Move, MAX_LEGAL_MOVES>::new();
        expected_moves.try_extend_from_slice(&[Move::new(27, 20, MoveType::EnPassant), Move::new(29, 20, MoveType::EnPassant)]).unwrap();
        expected_moves.sort();
        assert_eq!(moves, expected_moves);
        //let board = Board::from_fen("5n1n/6P1/8/8/8/8/8/k3K3 w - - 0 1");
        //let mut moves = ArrayVec::<Move, MAX_LEGAL_MOVES>::new();
        //generate_pawn_moves(moves, &board);
    }
    #[test]
    fn test_knight_moves() {
        let board = Board::start_pos();
        let mut moves = ArrayVec::<Move, MAX_LEGAL_MOVES>::new();
        generate_knight_moves(&mut moves, &board);
        assert_eq!(moves.len(), 4);
    }
    #[test]
    fn test_move_legality() {
        let board = Board::from_fen("4k3/8/4q3/8/8/4R3/8/4K3 w - - 0 1");
        assert!(legal(&board, 44, 36));
        assert!(legal(&board, 44, 28));
        assert!(legal(&board, 44, 52));
        assert!(!legal(&board, 44, 43));
        assert!(!legal(&board, 44, 42));
        assert!(!legal(&board, 44, 45));
    }
    #[test]
    fn test_resolve_single_check() {
        let board = Board::from_fen("rnb1kbnr/ppppqppp/8/8/8/8/3P1P2/4KN2 w kq - 0 1");
        let mut expected_moves = ArrayVec::<Move, MAX_LEGAL_MOVES>::new();
        expected_moves.try_extend_from_slice(&[Move::new(61, 44, MoveType::Normal), Move::new(60, 59, MoveType::Normal)]).unwrap();
        expected_moves.sort();
        let mut moves = ArrayVec::<Move, MAX_LEGAL_MOVES>::new();
        generate_king_moves(&mut moves, &board);
        resolve_single_check::<true>(12, &mut moves, &board);
        moves.sort();
        assert_eq!(moves, expected_moves);

        let board = Board::from_fen("rnb1kbnr/ppppqppp/8/8/1B6/8/3P1P2/3RKR2 w kq - 0 1");
        let mut expected_moves = ArrayVec::<Move, MAX_LEGAL_MOVES>::new();
        expected_moves.try_extend_from_slice(&[Move::new(33, 12, MoveType::Normal)]).unwrap();
        expected_moves.sort();
        let mut moves = ArrayVec::<Move, MAX_LEGAL_MOVES>::new();
        generate_king_moves(&mut moves, &board);
        resolve_single_check::<true>(12, &mut moves, &board);
        moves.sort();
        assert_eq!(moves, expected_moves);

        let board = Board::from_fen("2k5/8/3b4/2PPpP2/2PKP3/2PPP3/8/8 w - e6 0 1");
        let mut expected_moves = ArrayVec::<Move, MAX_LEGAL_MOVES>::new();
        expected_moves.try_extend_from_slice(&[Move::new(27, 20, MoveType::EnPassant), Move::new(29, 20, MoveType::EnPassant)]).unwrap();
        expected_moves.sort();
        let mut moves = ArrayVec::<Move, MAX_LEGAL_MOVES>::new();
        generate_king_moves(&mut moves, &board);
        resolve_single_check::<true>(28, &mut moves, &board);
        moves.sort();
        assert_eq!(moves, expected_moves);

        let board = Board::from_fen("6k1/8/1b6/8/2P5/8/3P2PP/5NKR w - - 0 1");
        let mut expected_moves = ArrayVec::<Move, MAX_LEGAL_MOVES>::new();
        expected_moves.try_extend_from_slice(&[Move::new(61, 44, MoveType::Normal), Move::new(34, 26, MoveType::Normal), Move::new(51, 35, MoveType::DoublePush)]).unwrap();
        expected_moves.sort();
        let mut moves = ArrayVec::<Move, MAX_LEGAL_MOVES>::new();
        generate_king_moves(&mut moves, &board);
        resolve_single_check::<true>(17, &mut moves, &board);
        moves.sort();
        assert_eq!(moves, expected_moves);
    }
    #[test]
    fn test_castling() {
        let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        let mut expected_moves = ArrayVec::<Move, MAX_LEGAL_MOVES>::new();
        expected_moves.try_extend_from_slice(&[Move::new(60, 62, MoveType::KingsideCastle), Move::new(60, 58, MoveType::QueensideCastle)]).unwrap();
        expected_moves.sort();
        let mut moves = ArrayVec::<Move, MAX_LEGAL_MOVES>::new();
        generate_castling_moves::<true>(&mut moves, &board);
        moves.sort();
        assert_eq!(moves, expected_moves);
    }
}
