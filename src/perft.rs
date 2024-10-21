use crate::board::bitboard::Bitboard;
use crate::board::piece::{Piece, PieceType};
use crate::board::piece_move::{Move, MoveType};
use crate::board::Board;
use crate::move_generation::attack_tables::BETWEEN_RAYS;
use crate::move_generation::generate_moves;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Add;

#[derive(Default, Clone, Copy)]
pub struct PerftResult {
    pub nodes: u64,
    pub captures: u32,
    pub en_passants: u32,
    pub castles: u32,
    pub promotions: u32,
    pub checks: u32,
    pub discovered_checks: u32,
    pub double_checks: u32,
    pub checkmates: u32,
}

impl Add for PerftResult {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            nodes: self.nodes + rhs.nodes,
            captures: self.captures + rhs.captures,
            en_passants: self.en_passants + rhs.en_passants,
            castles: self.castles + rhs.castles,
            promotions: self.promotions + rhs.promotions,
            checks: self.checks + rhs.checks,
            discovered_checks: self.discovered_checks + rhs.discovered_checks,
            double_checks: self.double_checks + rhs.double_checks,
            checkmates: self.checkmates + rhs.checkmates,
        }
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
        writeln!(f, "Discovered Checks: {}", self.discovered_checks).unwrap();
        writeln!(f, "Double Checks: {}", self.double_checks).unwrap();
        writeln!(f, "Checkmates: {}", self.checkmates).unwrap();
        Ok(())
    }
}

pub fn perft(fen: &str, depth: u32) -> PerftResult {
    let mut move_counter = HashMap::<Move, u64>::new();
    let mut result = PerftResult::default();
    let mut board = Board::from_fen(fen);
    for mov in generate_moves(&board) {
        board.make_move(mov);
        let nodes = search(depth - 1, mov, &mut board);
        board.unmake_move(mov);

        result = result + nodes;
        move_counter.insert(mov, nodes.nodes);
    }
    let mut sorted_keys: Vec<Move> = move_counter.keys().copied().collect();
    sorted_keys.sort();
    let mut sorted_moves = HashMap::new();
    for key in sorted_keys {
        if let Some(value) = move_counter.get(&key) {
            sorted_moves.insert(key, *value);
        }
    }
    sorted_moves.iter().for_each(|pair| println!("{}: {}", pair.0, pair.1));
    result
}

fn search(depth: u32, prev_mov: Move, board: &mut Board) -> PerftResult {
    let moves = generate_moves(board);

    //if depth == 0 {
    //    return PerftResult { nodes: 1 , ..Default::default() };
    //}

    if depth == 1 {
        return PerftResult { nodes: moves.len() as u64, ..Default::default() };
    }

    let mut result = PerftResult::default();

    for mov in moves {
        board.make_move(mov);
        result = result + search(depth - 1, mov, board);
        board.unmake_move(mov);
    }
    result
}

fn get_move_info(mov: Move, board: &Board, extra_info: bool) -> PerftResult {
    let mut info = PerftResult { nodes: 1, ..Default::default() };

    if extra_info {
        if board.state().captured_piece.is_some() {
            info.captures = 1;
        }
        if mov.move_type() == MoveType::EnPassant {
            info.en_passants = 1;
        }
        if mov.move_type() == MoveType::KingsideCastle || mov.move_type() == MoveType::QueensideCastle {
            info.castles = 1;
        }
        if MoveType::PROMOTIONS.contains(&mov.move_type()) {
            info.promotions = 1;
        }
        let king_square = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();
        let mut attackers = board.attackers(king_square, board.side);
        if attackers.count_ones() > 0 {
            info.checks = 1;
            while attackers != 0 {
                let attacker_square = attackers.pop_lsb();
                if Bitboard::from_square(mov.from()) & BETWEEN_RAYS[attacker_square][king_square] != 0 {
                    info.discovered_checks = 1;
                }
            }
            if attackers == 2 {
                info.double_checks = 1;
            }
        }
        if generate_moves(board).is_empty() {
            info.checkmates = 1;
        }
    }

    info
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn test_perft_startpos() {
        let start = Instant::now();
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

        assert_eq!(perft(fen, 1).nodes, 20);
        assert_eq!(perft(fen, 2).nodes, 400);
        assert_eq!(perft(fen, 3).nodes, 8902);
        assert_eq!(perft(fen, 4).nodes, 197281);
        //assert_eq!(perft(fen, 5).nodes, 4865609);
        //assert_eq!(perft(fen, 6).nodes, 119060324);
        //assert_eq!(perft(fen, 7).nodes, 3195901860);

        // Calculate elapsed time
        let elapsed = start.elapsed();

        // Print elapsed time
        println!("Test took {} milliseconds", elapsed.as_millis());
    }
    #[test]
    fn test_perft2() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        assert_eq!(perft(fen, 1).nodes, 48);
        assert_eq!(perft(fen, 2).nodes, 2039);
        assert_eq!(perft(fen, 3).nodes, 97862);
        assert_eq!(perft(fen, 4).nodes, 4085603);
        //assert_eq!(perft(fen, 5).nodes, 193690690);
    }
    #[test]
    fn test_perft3() {
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        assert_eq!(perft(fen, 1).nodes, 14);
        assert_eq!(perft(fen, 2).nodes, 191);
        assert_eq!(perft(fen, 3).nodes, 2812);
        assert_eq!(perft(fen, 4).nodes, 43238);
        //assert_eq!(perft(fen, 5).nodes, 674624);
    }
    #[test]
    fn test_perft4() {
        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        assert_eq!(perft(fen, 1).nodes, 6);
        assert_eq!(perft(fen, 2).nodes, 264);
        assert_eq!(perft(fen, 3).nodes, 9467);
        assert_eq!(perft(fen, 4).nodes, 422333);
        //assert_eq!(perft(fen, 5).nodes, 15833292);
    }
    #[test]
    fn test_perft5() {
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        assert_eq!(perft(fen, 1).nodes, 44);
        assert_eq!(perft(fen, 2).nodes, 1486);
        assert_eq!(perft(fen, 3).nodes, 62379);
        assert_eq!(perft(fen, 4).nodes, 2103487);
        //assert_eq!(perft(fen, 5).nodes, 89941194);
    }
    #[test]
    fn test_perft6() {
        let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
        assert_eq!(perft(fen, 1).nodes, 46);
        assert_eq!(perft(fen, 2).nodes, 2079);
        assert_eq!(perft(fen, 3).nodes, 89890);
        assert_eq!(perft(fen, 4).nodes, 3894594);
        //assert_eq!(perft(fen, 5).nodes, 164075551);
        //assert_eq!(perft(fen, 6).nodes, 6923051137);
    }
}
