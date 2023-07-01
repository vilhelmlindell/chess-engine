use std::fmt::{Display, Formatter};
use std::ops::Add;
use crate::board::Board;
use crate::piece::{Piece, PieceType};
use crate::piece_move::{Move, MoveType};

pub struct PerftResult {
    nodes: u32,
    captures: u32,
    en_passants: u32,
    castles: u32,
    promotions: u32,
    checks: u32,
    checkmates: u32,
}

impl Default for PerftResult {
    fn default() -> Self {
        Self {
            nodes: 0,
            captures: 0,
            en_passants: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
            checkmates: 0,
        }
    }
}
impl Add for PerftResult {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut sum = PerftResult::default();
        sum.nodes = self.nodes + rhs.nodes;
        sum.captures = self.captures + rhs.captures;
        sum.en_passants = self.en_passants + rhs.en_passants;
        sum.castles = self.castles + rhs.castles;
        sum.promotions = self.promotions + rhs.promotions;
        sum.checks = self.checks + rhs.checks;
        sum.checkmates = self.checkmates + rhs.checkmates;
        sum
    }
}
impl Display for PerftResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Nodes: {}", self.nodes).unwrap();
        writeln!(f, "Captures: {}", self.captures).unwrap();
        writeln!(f, "En Passants: {}", self.en_passants).unwrap();
        writeln!(f, "Castles: {}", self.castles).unwrap();
        writeln!(f, "Promotions: {}", self.promotions).unwrap();
        writeln!(f, "Checks: {}", self.checks).unwrap();
        writeln!(f, "Checkmates: {}", self.checkmates).unwrap();
        Ok(())
    }
}

pub fn perft(depth: u32) -> PerftResult {
    let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
    println!("{}", board);
    let result = search(depth, None, &mut board);
    println!("{}", board);
    result
}

fn search(depth: u32, current_move: Option<Move>, board: &mut Board) -> PerftResult {
    if depth == 0 {
        return get_move_info(current_move.unwrap(), board);
    }
    let mut result = PerftResult::default();
    for mov in board.generate_moves() {
        board.make_move(mov);
        result = result + search(depth - 1, Some(mov), board);
        board.unmake_move(mov);
    }
    result
}

fn get_move_info(mov: Move, board: &mut Board) -> PerftResult {
    let mut info = PerftResult::default();
    info.nodes = 1;

    if board.state().captured_piece.is_some() {
        info.captures = 1;
        return info;
    }
    if mov.move_type == MoveType::EnPassant {
        info.en_passants = 1;
        return info;
    }
    if let MoveType::Castle {kingside: _} = mov.move_type {
        info.castles = 1;
        return info;
    }
    if let MoveType::Promotion(_) = mov.move_type {
        info.promotions = 1;
        return info;
    }
    if let MoveType::Promotion(_) = mov.move_type {
        info.promotions = 1;
        return info;
    }
    let king_square = board.piece_squares[Piece::new(PieceType::King, board.side_to_move)].lsb();
    if board.attacked(king_square) {
        info.checks = 1;
        return info;
    }

    info
}
