use crate::attack_tables::*;
use crate::bitboard::Bitboard;
use crate::direction::Direction;
use crate::piece::{Piece, PieceType};
use crate::piece_move::{Move, MoveType};
use crate::piece_square_tables::position_value;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Index, IndexMut};

const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn square_from_string(square: String) -> usize {
    let files = "abcdefgh";
    let rank = files.chars().position(|char| square.as_bytes()[0] as char == char).unwrap();
    let file = (8 - (square.as_bytes()[1] as char).to_digit(10).unwrap()) as usize;
    file * 8 + rank
}

#[non_exhaustive]
#[derive(Clone)]
pub struct Board {
    pub squares: [Option<Piece>; 64],
    pub side: Side,
    pub occupied_squares: Bitboard,
    pub side_squares: [Bitboard; 2],
    pub attacked_squares: [Bitboard; 2],
    pub piece_squares: [Bitboard; 12],
    pub absolute_pinned_squares: Bitboard,
    pub states: Vec<BoardState>,
    pub material_balance: i32,
    pub position_balance: i32,
}

impl Board {
    pub fn new() -> Self {
        Self {
            squares: [Option::<Piece>::None; 64],
            side: Side::White,
            occupied_squares: Bitboard(0),
            side_squares: [Bitboard(0); 2],
            attacked_squares: [Bitboard(0); 2],
            piece_squares: [Bitboard(0); 12],
            absolute_pinned_squares: Bitboard(0),
            states: vec![BoardState::default()],
            material_balance: 0,
            position_balance: 0,
        }
    }
    pub fn from_fen(fen: &str) -> Self {
        let mut board = Self::new();
        board.load_fen(fen);
        board.absolute_pinned_squares = board.absolute_pins();
        board
    }
    pub fn fen(&self) -> String {
        let mut fen = "".to_string();
        let piece_types = HashMap::from([
            (PieceType::Pawn, 'p'),
            (PieceType::Knight, 'n'),
            (PieceType::Bishop, 'b'),
            (PieceType::Rook, 'r'),
            (PieceType::Queen, 'q'),
            (PieceType::King, 'k'),
        ]);
        for rank in 0..8 {
            let mut empty = 0;
            for file in 0..8 {
                let square = rank * 8 + file;
                if let Some(piece) = self.squares[square] {
                    if empty != 0 {
                        fen.push(char::from_digit(empty, 10).unwrap());
                    }
                    let mut piece_char = *piece_types.get(&piece.piece_type()).unwrap();
                    if piece.side() == Side::White {
                        piece_char = piece_char.to_ascii_uppercase();
                    }
                    fen.push(piece_char);
                    empty = 0;
                } else {
                    empty += 1;
                    if file == 7 {
                        fen.push(char::from_digit(empty, 10).unwrap());
                    }
                }
            }
            if rank != 7 {
                fen.push('/');
            }
        }
        fen.push(' ');
        match self.side {
            Side::White => fen.push('w'),
            Side::Black => fen.push('b'),
        };
        fen.push(' ');
        if self.state().castling_rights[Side::White].kingside {
            fen.push('K');
        }
        if self.state().castling_rights[Side::White].queenside {
            fen.push('Q');
        }
        if self.state().castling_rights[Side::Black].kingside {
            fen.push('k');
        }
        if self.state().castling_rights[Side::Black].kingside {
            fen.push('q');
        }
        fen.push(' ');
        if let Some(square) = self.state().en_passant_square {
            let mut square_string = "".to_string();
            let files = "abcdefgh".to_string();
            let rank = square / 8;
            let file = square % 8;
            square_string.push(*files.as_bytes().get(file).unwrap() as char);
            square_string.push(char::from_digit(8 - rank as u32, 10).unwrap());
            fen.push_str(&square_string);
        } else {
            fen.push('-');
        }
        fen.push_str(" 0 1");
        fen
    }
    pub fn start_pos() -> Self {
        Self::from_fen(STARTING_FEN)
    }

    pub fn friendly_squares(&self) -> Bitboard {
        self.side_squares[self.side]
    }
    pub fn enemy_squares(&self) -> Bitboard {
        self.side_squares[self.side.enemy()]
    }
    pub fn state(&self) -> &BoardState {
        self.states.last().unwrap()
    }
    pub fn state_mut(&mut self) -> &mut BoardState {
        self.states.last_mut().unwrap()
    }
    pub fn is_capture(&self, mov: Move) -> bool {
        Option::is_some(&self.squares[mov.to])
    }

    pub fn load_fen(&mut self, fen: &str) {
        let piece_types = HashMap::from([
            ('p', PieceType::Pawn),
            ('n', PieceType::Knight),
            ('b', PieceType::Bishop),
            ('r', PieceType::Rook),
            ('q', PieceType::Queen),
            ('k', PieceType::King),
        ]);
        let fields: Vec<&str> = fen.split(' ').collect();
        let ranks: Vec<&str> = fields.first().unwrap().split('/').collect();
        for (rank, rank_string) in ranks.iter().enumerate() {
            let mut file = 0;
            for i in 0..rank_string.len() {
                let piece_char = rank_string.as_bytes()[i] as char;
                if piece_char.is_numeric() {
                    let piece_char: char = ranks[rank].as_bytes()[i] as char;
                    file += piece_char.to_digit(10).unwrap() as usize;
                } else {
                    let square = rank * 8 + file;
                    if piece_char.is_uppercase() {
                        self.squares[square] = Some(Piece::new(piece_types.get(&piece_char.to_ascii_lowercase()).copied().unwrap(), Side::White))
                    } else {
                        self.squares[square] = Some(Piece::new(piece_types.get(&piece_char).copied().unwrap(), Side::Black))
                    }
                }
                if !piece_char.is_numeric() {
                    file += 1;
                }
            }
        }
        // Set side to move
        match *fields.get(1).unwrap() {
            "w" => self.side = Side::White,
            "b" => self.side = Side::Black,
            _ => panic!("Invalid fen string"),
        }
        // Set castling rights
        self.state_mut().castling_rights = [CastlingRights::default(); 2];
        for castling_right in fields.get(2).unwrap().chars() {
            match castling_right {
                'K' => self.state_mut().castling_rights[Side::White].kingside = true,
                'Q' => self.state_mut().castling_rights[Side::White].queenside = true,
                'k' => self.state_mut().castling_rights[Side::Black].kingside = true,
                'q' => self.state_mut().castling_rights[Side::Black].queenside = true,
                '-' => {}
                _ => panic!("Invalid fen string"),
            }
        }

        match *fields.get(3).unwrap() {
            "-" => {}
            _ => self.state_mut().en_passant_square = Some(square_from_string(fields.get(3).unwrap().to_string())),
        }

        self.initialize_bitboards();
        for square in 0..64 {
            if let Some(piece) = self.squares[square] {
                self.material_balance += piece.piece_type().centipawns() * piece.side().factor();
                self.position_balance += position_value(piece.piece_type(), square, piece.side()) * piece.side().factor()
            }
        }
    }
    pub fn make_move(&mut self, mov: Move) {
        let mut state = BoardState::from_state(self.state());

        if mov.from == 0 || mov.from == 4 || mov.to == 0 {
            state.castling_rights[Side::Black].queenside = false;
        }
        if mov.from == 7 || mov.from == 4 || mov.to == 7 {
            state.castling_rights[Side::Black].kingside = false;
        }
        if mov.from == 56 || mov.from == 60 || mov.to == 56 {
            state.castling_rights[Side::White].queenside = false;
        }
        if mov.from == 63 || mov.from == 60 || mov.to == 63 {
            state.castling_rights[Side::White].kingside = false;
        }

        if let Some(captured_piece) = self.squares[mov.to] {
            self.clear_square(mov.to);
            state.captured_piece = Some(captured_piece);
        }

        self.move_piece(mov.from, mov.to);

        match mov.move_type {
            MoveType::Normal => {}
            MoveType::Castle { kingside } => {
                let (rook_from, rook_to) = match (self.side, kingside) {
                    (Side::White, true) => (63, 61),
                    (Side::White, false) => (56, 59),
                    (Side::Black, true) => (7, 5),
                    (Side::Black, false) => (0, 3),
                };
                self.move_piece(rook_from, rook_to);
            }
            MoveType::DoublePush => {
                state.en_passant_square = Some((mov.to as i32 + Direction::down(self.side).value()) as usize);
            }
            MoveType::Promotion(piece_type) => {
                let piece = Piece::new(piece_type, self.side);
                self.clear_square(mov.to);
                self.set_square(mov.to, piece);
            }
            MoveType::EnPassant => {
                let en_passant_square = self.state().en_passant_square.unwrap();
                let capture_square = (en_passant_square as i32 + Direction::down(self.side).value()) as usize;
                let captured_piece = self.squares[capture_square].unwrap();
                self.clear_square(capture_square);
                state.captured_piece = Some(captured_piece);
            }
        }

        self.side = self.side.enemy();
        self.absolute_pinned_squares = self.absolute_pins();
        self.states.push(state);
    }
    pub fn unmake_move(&mut self, mov: Move) {
        self.side = self.side.enemy();

        self.move_piece(mov.to, mov.from);

        // Restore the captured piece, if any
        if mov.move_type != MoveType::EnPassant {
            if let Some(piece) = self.state_mut().captured_piece {
                self.set_square(mov.to, piece);
            }
        }

        // Undo castling, if necessary
        if let MoveType::Castle { kingside } = mov.move_type {
            let (rook_from, rook_to) = match (self.side, kingside) {
                (Side::White, true) => (61, 63),
                (Side::White, false) => (59, 56),
                (Side::Black, true) => (5, 7),
                (Side::Black, false) => (3, 0),
            };
            self.move_piece(rook_from, rook_to);
        }

        // Restore a pawn that was promoted to a non-pawn piece
        if let MoveType::Promotion(_) = mov.move_type {
            let pawn = Piece::new(PieceType::Pawn, self.side);
            self.clear_square(mov.from);
            self.set_square(mov.from, pawn);
        }

        // Restore a captured pawn in an en passant capture
        if mov.move_type == MoveType::EnPassant {
            let square = (self.states[self.states.len() - 2].en_passant_square.unwrap() as i32 + Direction::down(self.side).value()) as usize;
            let captured_pawn = Piece::new(PieceType::Pawn, self.side.enemy());
            self.set_square(square, captured_pawn);
            //self.material_balance -= captured_pawn.piece_type().centipawns();
        }

        //self.material_balance += material_change * self.side_to_move.factor();
        self.absolute_pinned_squares = self.absolute_pins();
        self.states.pop();
    }
    pub fn attacked(&self, square: usize) -> bool {
        let pawns = self.piece_squares[Piece::new(PieceType::Pawn, self.side.enemy())];
        if (PAWN_ATTACKS[self.side.enemy()][square] & pawns).0 != 0 {
            return true;
        }
        let knights = self.piece_squares[Piece::new(PieceType::Knight, self.side.enemy())];
        if (KNIGHT_ATTACK_MASKS[square] & knights).0 != 0 {
            return true;
        }
        let queens = self.piece_squares[Piece::new(PieceType::Queen, self.side.enemy())];
        let bishops = self.piece_squares[Piece::new(PieceType::Bishop, self.side.enemy())];
        if (bishop_attacks(square, self.occupied_squares) & (bishops | queens)).0 != 0 {
            return true;
        }
        let rooks = self.piece_squares[Piece::new(PieceType::Rook, self.side.enemy())];
        if (rook_attacks(square, self.occupied_squares) & (rooks | queens)).0 != 0 {
            return true;
        }
        false
    }
    #[inline(always)]
    pub fn attackers(&self, square: usize, side: Side) -> Bitboard {
        let mut attackers = Bitboard(0);

        let pawns = self.piece_squares[Piece::new(PieceType::Pawn, side.enemy())];
        attackers |= PAWN_ATTACKS[side.enemy()][square] & pawns;

        let knights = self.piece_squares[Piece::new(PieceType::Knight, side.enemy())];
        attackers |= KNIGHT_ATTACK_MASKS[square] & knights;

        let queens = self.piece_squares[Piece::new(PieceType::Queen, side.enemy())];

        let bishops = self.piece_squares[Piece::new(PieceType::Bishop, side.enemy())];
        attackers |= bishop_attacks(square, self.occupied_squares) & (bishops | queens);

        let rooks = self.piece_squares[Piece::new(PieceType::Rook, side.enemy())];
        attackers |= rook_attacks(square, self.occupied_squares) & (rooks | queens);

        attackers
    }
    pub fn king_attacked(&self, from: usize, to: usize) -> bool {
        let pawns = self.piece_squares[Piece::new(PieceType::Pawn, self.side.enemy())];
        if (PAWN_ATTACKS[self.side.enemy()][to] & pawns).0 != 0 {
            return true;
        }
        let knights = self.piece_squares[Piece::new(PieceType::Knight, self.side.enemy())];
        if (KNIGHT_ATTACK_MASKS[to] & knights).0 != 0 {
            return true;
        }
        let queens = self.piece_squares[Piece::new(PieceType::Queen, self.side.enemy())];
        let bishops = self.piece_squares[Piece::new(PieceType::Bishop, self.side.enemy())];
        let occupied = self.occupied_squares ^ Bitboard::from_square(from);
        if (bishop_attacks(to, occupied) & (bishops | queens)).0 != 0 {
            return true;
        }
        let rooks = self.piece_squares[Piece::new(PieceType::Rook, self.side.enemy())];
        if (rook_attacks(to, occupied) & (rooks | queens)).0 != 0 {
            return true;
        }
        false
    }
    #[inline(always)]
    pub fn aligned(square1: usize, square2: usize, square3: usize) -> bool {
        LINE_RAYS[square1][square2] & Bitboard::from_square(square3) != 0
    }

    fn initialize_bitboards(&mut self) {
        for square in 0..64 {
            if let Some(piece) = self.squares[square] {
                self.piece_squares[piece].set_bit(square);
                self.side_squares[piece.side()].set_bit(square);
                self.occupied_squares.set_bit(square);
            } else {
                self.piece_squares.iter_mut().for_each(|x| x.clear_bit(square));
                self.side_squares.iter_mut().for_each(|x| x.clear_bit(square));
                self.occupied_squares.clear_bit(square);
            }
        }
    }
    #[inline(always)]
    fn move_piece(&mut self, from: usize, to: usize) {
        let piece = self.squares[from].unwrap();
        self.set_square(to, piece);
        self.clear_square(from);
    }

    #[inline(always)]
    fn set_square(&mut self, square: usize, piece: Piece) {
        self.occupied_squares.set_bit(square);
        self.side_squares[piece.side()].set_bit(square); // Update kingside castling right
        self.piece_squares[piece].set_bit(square);
        self.squares[square] = Some(piece);
        self.position_balance += position_value(piece.piece_type(), square, piece.side()) * piece.side().factor();
        self.material_balance += piece.piece_type().centipawns() * piece.side().factor();
    }
    #[inline(always)]
    fn clear_square(&mut self, square: usize) {
        let piece = self.squares[square].unwrap();
        self.occupied_squares.clear_bit(square);
        self.side_squares[piece.side()].clear_bit(square);
        self.piece_squares[piece].clear_bit(square);
        self.squares[square] = None;
        self.position_balance -= position_value(piece.piece_type(), square, piece.side()) * piece.side().factor();
        self.material_balance -= piece.piece_type().centipawns() * piece.side().factor();
    }
    #[inline(always)]
    fn absolute_pins(&self) -> Bitboard {
        let king_square = self.piece_squares[Piece::new(PieceType::King, self.side)].lsb();

        let mut pinned_squares = Bitboard(0);
        let mut pinners = self.xray_rook_attacks(king_square, self.friendly_squares())
            & (self.piece_squares[Piece::new(PieceType::Rook, self.side.enemy())] | self.piece_squares[Piece::new(PieceType::Queen, self.side.enemy())]);

        while pinners != 0 {
            let pinner_square = pinners.pop_lsb();
            pinned_squares |= BETWEEN_RAYS[king_square][pinner_square] & self.friendly_squares();
        }

        pinners = self.xray_bishop_attacks(king_square, self.friendly_squares())
            & (self.piece_squares[Piece::new(PieceType::Bishop, self.side.enemy())] | self.piece_squares[Piece::new(PieceType::Queen, self.side.enemy())]);

        while pinners != 0 {
            let pinner_square = pinners.pop_lsb();
            pinned_squares |= BETWEEN_RAYS[king_square][pinner_square] & self.friendly_squares();
        }

        pinned_squares
    }
    #[inline(always)]
    fn xray_rook_attacks(&self, square: usize, blockers: Bitboard) -> Bitboard {
        let attacks = rook_attacks(square, self.occupied_squares);
        let attacked_blockers = blockers & attacks;
        attacks ^ rook_attacks(square, self.occupied_squares ^ attacked_blockers)
    }
    #[inline(always)]
    pub fn xray_bishop_attacks(&self, square: usize, blockers: Bitboard) -> Bitboard {
        let attacks = bishop_attacks(square, self.occupied_squares);
        let attacked_blockers = blockers & attacks;
        attacks ^ bishop_attacks(square, self.occupied_squares ^ attacked_blockers)
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece_chars = HashMap::from([
            (PieceType::Pawn, 'p'),
            (PieceType::Knight, 'n'),
            (PieceType::Bishop, 'b'),
            (PieceType::Rook, 'r'),
            (PieceType::Queen, 'q'),
            (PieceType::King, 'k'),
        ]);
        for rank in 0..8 {
            write!(f, "{}", 8 - rank).unwrap();
            for file in 0..8 {
                write!(f, "{}", ' ').unwrap();
                match self.squares[rank * 8 + file] {
                    Some(piece) => {
                        let piece_char = piece_chars.get(&piece.piece_type()).unwrap();
                        match piece.side() {
                            Side::White => write!(f, "{}", piece_char.to_ascii_uppercase()).unwrap(),
                            Side::Black => write!(f, "{}", piece_char).unwrap(),
                        }
                    }
                    None => write!(f, "{}", '.').unwrap(),
                }
            }
            writeln!(f).unwrap();
        }
        write!(f, "{}", ' ').unwrap();
        for file in 'a'..='h' {
            write!(f, "{}", ' ').unwrap();
            write!(f, "{}", file).unwrap();
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Side {
    White = 0,
    Black,
}

impl Side {
    pub fn value(&self) -> u32 {
        *self as u32
    }
    pub fn enemy(&self) -> Side {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
    pub fn factor(&self) -> i32 {
        match self {
            Side::White => 1,
            Side::Black => -1,
        }
    }
}
impl<T, const N: usize> Index<Side> for [T; N] {
    type Output = T;

    fn index(&self, index: Side) -> &Self::Output {
        &self[index as usize]
    }
}
impl<T, const N: usize> IndexMut<Side> for [T; N] {
    fn index_mut(&mut self, index: Side) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

#[derive(Clone, Copy)]
pub struct BoardState {
    // Copied
    pub halfmove_clock: u32,

    // Recalculated
    pub castling_rights: [CastlingRights; 2],
    pub en_passant_square: Option<usize>,
    pub captured_piece: Option<Piece>,
}

impl BoardState {
    pub fn from_state(state: &Self) -> Self {
        Self {
            halfmove_clock: state.halfmove_clock,
            castling_rights: state.castling_rights,
            ..Self::default()
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct CastlingRights {
    pub kingside: bool,
    pub queenside: bool,
}

impl Default for BoardState {
    fn default() -> Self {
        Self {
            halfmove_clock: 0,
            castling_rights: [CastlingRights { kingside: false, queenside: false }, CastlingRights { kingside: false, queenside: false }],
            en_passant_square: None,
            captured_piece: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_fen() {
        let mut squares = [Option::<Piece>::None; 64];
        for i in 0..8 {
            squares[8 + i] = Some(Piece::new(PieceType::Pawn, Side::Black));
            squares[48 + i] = Some(Piece::new(PieceType::Pawn, Side::White));
        }
        squares[0] = Some(Piece::new(PieceType::Rook, Side::Black));
        squares[1] = Some(Piece::new(PieceType::Knight, Side::Black));
        squares[2] = Some(Piece::new(PieceType::Bishop, Side::Black));
        squares[3] = Some(Piece::new(PieceType::Queen, Side::Black));
        squares[4] = Some(Piece::new(PieceType::King, Side::Black));
        squares[5] = Some(Piece::new(PieceType::Bishop, Side::Black));
        squares[6] = Some(Piece::new(PieceType::Knight, Side::Black));
        squares[7] = Some(Piece::new(PieceType::Rook, Side::Black));

        squares[56] = Some(Piece::new(PieceType::Rook, Side::White));
        squares[57] = Some(Piece::new(PieceType::Knight, Side::White));
        squares[58] = Some(Piece::new(PieceType::Bishop, Side::White));
        squares[59] = Some(Piece::new(PieceType::Queen, Side::White));
        squares[60] = Some(Piece::new(PieceType::King, Side::White));
        squares[61] = Some(Piece::new(PieceType::Bishop, Side::White));
        squares[62] = Some(Piece::new(PieceType::Knight, Side::White));
        squares[63] = Some(Piece::new(PieceType::Rook, Side::White));

        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - -");
        assert_eq!(squares, board.squares);
    }
    #[test]
    fn sets_correct_bitboards_from_squares() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - -");
        let white_pawn_bitboard = board.piece_squares[Piece::new(PieceType::Pawn, Side::White)];
        assert_eq!(white_pawn_bitboard.0, 0x00FF000000000000)
    }
    #[test]
    fn test_aligned() {
        assert!(Board::aligned(28, 44, 60));
        assert!(!Board::aligned(43, 44, 60));
    }
    #[test]
    fn test_pawn_moves() {
        let mut board = Board::from_fen("8/8/3p1p2/3PpP2/8/1k6/2p5/Kn6 w - e6 0 1");
        let mov = Move::new(27, 20, MoveType::EnPassant);
        //let mov2 = Move::new(29, 20, MoveType::EnPassant);
        println!("{board}");
        board.make_move(mov);
        board.unmake_move(mov);
        println!("{board}");
        assert!(true);
    }
    #[test]
    fn make_unmake() {
        let original = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/1B1PN3/1p2P3/2N2Q1p/PPPB1PPP/R3K2R b KQkq - 1 1");
        let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/1B1PN3/1p2P3/2N2Q1p/PPPB1PPP/R3K2R b KQkq - 1 1");
        for mov in board.generate_moves() {
            board.make_move(mov);
            board.unmake_move(mov);
            println!();
            println!("{original}");
            println!("{board}");

            for square in 0..64 {
                assert!(original.squares[square] == board.squares[square]);
                assert!(original.absolute_pins() == board.absolute_pins());
            }
        }
    }
}
