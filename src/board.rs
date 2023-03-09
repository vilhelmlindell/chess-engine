use crate::bitboard::Bitboard;
use crate::direction::Direction;
use crate::piece::{Piece, PieceType};
use crate::piece_move::{Move, MoveType};
use crate::piece_square_tables::get_positional_value;
use crate::tables::*;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Index, IndexMut};

const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

pub const WHITE_KINGSIDE_CASTLING_MASK: Bitboard = Bitboard(0x6000000000000000);
pub const WHITE_QUEENSIDE_CASTLING_MASK: Bitboard = Bitboard(0x0E00000000000000);
pub const BLACK_KINGSIDE_CASTLING_MASK: Bitboard = Bitboard(0x0000000000000060);
pub const BLACK_QUEENSIDE_CASTLING_MASK: Bitboard = Bitboard(0x000000000000000E);

#[non_exhaustive]
pub struct Board {
    pub squares: [Option<Piece>; 64],
    pub side_to_move: Side,
    pub occupied_squares: Bitboard,
    pub side_bitboards: [Bitboard; 2],
    pub attacked_squares: [Bitboard; 2],
    pub piece_bitboards: [Bitboard; 12],
    pub states: Vec<BoardState>,
    pub material_balance: i32,
    pub positional_balance: i32,
}

impl Board {
    pub fn new() -> Self {
        Self {
            squares: [Option::<Piece>::None; 64],
            side_to_move: Side::White,
            occupied_squares: Bitboard(0),
            side_bitboards: [Bitboard(0); 2],
            attacked_squares: [Bitboard(0); 2],
            piece_bitboards: [Bitboard(0); 12],
            states: vec![BoardState::default()],
            material_balance: 0,
            positional_balance: 0,
        }
    }
    pub fn from_fen(fen: &str) -> Self {
        let mut board = Self::new();
        board.load_fen(fen);
        board
    }
    pub fn start_pos() -> Self {
        Self::from_fen(STARTING_FEN)
    }

    pub fn friendly_squares(&self) -> Bitboard {
        self.side_bitboards[self.side_to_move]
    }
    pub fn enemy_squares(&self) -> Bitboard {
        self.side_bitboards[self.side_to_move.enemy()]
    }
    pub fn state(&self) -> BoardState {
        *self.states.last().unwrap()
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
        let ranks: Vec<&str> = fen.split('/').collect();
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
                        self.squares[square] = Some(Piece::new(&piece_types.get(&piece_char.to_ascii_lowercase()).copied().unwrap(), &Side::White))
                    } else {
                        self.squares[square] = Some(Piece::new(&piece_types.get(&piece_char).copied().unwrap(), &Side::Black))
                    }
                }
                if !piece_char.is_numeric() {
                    file += 1;
                }
            }
        }
        self.initialize_bitboards();
    }
    pub fn make_move(&mut self, mov: &Move) {
        let mut state = BoardState::from_state(&self.state());

        if let Some(captured_piece) = self.squares[mov.end_square] {
            self.clear_square(&mov.end_square);
            self.material_balance += captured_piece.piece_type().centipawns() * self.side_to_move.factor();
            state.captured_piece = Some(captured_piece);
        }

        self.move_piece(&mov.start_square, &mov.end_square);

        match mov.move_type {
            MoveType::Normal => {}
            MoveType::Castle { kingside } => match self.side_to_move {
                Side::White => {
                    if kingside {
                        self.move_piece(&63, &61);
                    } else {
                        self.move_piece(&56, &59);
                    }
                }
                Side::Black => {
                    if kingside {
                        self.move_piece(&7, &5);
                    } else {
                        self.move_piece(&0, &3);
                    }
                }
            },
            MoveType::DoublePush => {
                let down = match self.side_to_move {
                    Side::White => Direction::South,
                    Side::Black => Direction::North,
                };
                state.en_passant_square = Some((mov.end_square as i32 + down.value()) as usize);
            }
            MoveType::Promotion(piece_type) => {
                let piece = Piece::new(&piece_type, &self.side_to_move);
                self.clear_square(&mov.end_square);
                self.set_square(&mov.end_square, &piece);
            }
            MoveType::EnPassant => {
                let down: &Direction = match self.side_to_move {
                    Side::White => &Direction::South,
                    Side::Black => &Direction::North,
                };
                let square = (self.state().en_passant_square.unwrap() as i32 + down.value()) as usize;
                state.captured_piece = self.squares[square];
                self.material_balance += state.captured_piece.unwrap().piece_type().centipawns() * self.side_to_move.factor();
                self.clear_square(&square);
            }
        }
        state.castling_rights[self.side_to_move.enemy()] = CastlingRights { kingside: false, queenside: false };

        match self.side_to_move.enemy() {
            Side::White => {
                state.castling_rights[self.side_to_move].kingside =
                    WHITE_KINGSIDE_CASTLING_MASK & !self.occupied_squares == WHITE_KINGSIDE_CASTLING_MASK && !(self.is_attacked(60) || self.is_attacked(61) || self.is_attacked(62));
                state.castling_rights[self.side_to_move].queenside =
                    WHITE_QUEENSIDE_CASTLING_MASK & !self.occupied_squares == WHITE_QUEENSIDE_CASTLING_MASK && !(self.is_attacked(60) || self.is_attacked(59) || self.is_attacked(58))
            }
            Side::Black => {
                state.castling_rights[self.side_to_move].kingside =
                    BLACK_KINGSIDE_CASTLING_MASK & !self.occupied_squares == BLACK_KINGSIDE_CASTLING_MASK && !(self.is_attacked(4) || self.is_attacked(5) || self.is_attacked(6));
                state.castling_rights[self.side_to_move].queenside =
                    BLACK_QUEENSIDE_CASTLING_MASK & !self.occupied_squares == BLACK_QUEENSIDE_CASTLING_MASK && !(self.is_attacked(4) || self.is_attacked(3) || self.is_attacked(2));
            }
        }
        self.side_to_move = self.side_to_move.enemy();

        self.states.push(state);
    }
    pub fn unmake_move(&mut self, mov: &Move) {
        self.side_to_move = self.side_to_move.enemy();

        self.move_piece(&mov.end_square, &mov.start_square);

        if let Some(piece) = self.state().captured_piece {
            self.set_square(&mov.end_square, &piece);
            self.material_balance -= piece.piece_type().centipawns() * self.side_to_move.factor();
        }

        match mov.move_type {
            MoveType::Normal => {}
            MoveType::Castle { kingside } => match self.side_to_move {
                Side::White => {
                    if kingside {
                        self.move_piece(&61, &63);
                    } else {
                        self.move_piece(&59, &56);
                    }
                }
                Side::Black => {
                    if kingside {
                        self.move_piece(&5, &7);
                    } else {
                        self.move_piece(&3, &0);
                    }
                }
            },
            MoveType::DoublePush => {}
            MoveType::Promotion(_) => {
                self.set_square(&mov.start_square, &Piece::new(&PieceType::Pawn, &self.side_to_move.enemy()));
            }
            MoveType::EnPassant => {
                let down: &Direction = match self.side_to_move {
                    Side::White => &Direction::South,
                    Side::Black => &Direction::North,
                };

                let previous_state = self.states[self.states.len() - 2];
                let square = (previous_state.en_passant_square.unwrap() as i32 + down.value()) as usize;
                self.set_square(&square, &self.state().captured_piece.unwrap());
            }
        }

        self.states.pop();
    }

    fn initialize_bitboards(&mut self) {
        for square in 0..64 {
            if let Some(piece) = self.squares[square] {
                self.piece_bitboards[piece].set_bit(&square);
                self.side_bitboards[piece.side()].set_bit(&square);
                self.occupied_squares.set_bit(&square);
            } else {
                self.piece_bitboards.iter_mut().for_each(|x| x.clear_bit(&square));
                self.side_bitboards.iter_mut().for_each(|x| x.clear_bit(&square));
                self.occupied_squares.clear_bit(&square);
            }
        }
    }
    fn move_piece(&mut self, start_square: &usize, end_square: &usize) {
        let piece = self.squares[*start_square].unwrap();
        self.set_square(end_square, &piece);
        self.clear_square(start_square);
    }
    fn set_square(&mut self, square: &usize, piece: &Piece) {
        self.occupied_squares.set_bit(square);
        self.side_bitboards[piece.side()].set_bit(square);
        self.piece_bitboards[*piece].set_bit(square);
        self.squares[*square] = Some(*piece);
        self.positional_balance += get_positional_value(&piece.piece_type(), square, &piece.side()) * piece.side().factor();
    }
    fn clear_square(&mut self, square: &usize) {
        let piece = self.squares[*square].unwrap();
        self.occupied_squares.clear_bit(square);
        self.side_bitboards[piece.side()].clear_bit(square);
        self.piece_bitboards[piece].clear_bit(square);
        self.squares[*square] = None;
        self.positional_balance -= get_positional_value(&piece.piece_type(), square, &piece.side()) * piece.side().factor();
    }
    fn is_attacked(&self, square: usize) -> bool {
        let pawns = self.piece_bitboards[Piece::new(&PieceType::Pawn, &self.side_to_move.enemy())];
        if (PAWN_ATTACKS[self.side_to_move.enemy()][square] & pawns).0 != 0 {
            return true;
        }
        let knights = self.piece_bitboards[Piece::new(&PieceType::Knight, &self.side_to_move.enemy())];
        if (KNIGHT_ATTACK_MASKS[square] & knights).0 != 0 {
            return true;
        }
        let bishops = self.piece_bitboards[Piece::new(&PieceType::Bishop, &self.side_to_move.enemy())];
        if (get_bishop_attacks(&square, &self.occupied_squares) & bishops).0 != 0 {
            return true;
        }
        let rooks = self.piece_bitboards[Piece::new(&PieceType::Rook, &self.side_to_move.enemy())];
        if (get_rook_attacks(&square, &self.occupied_squares) & rooks).0 != 0 {
            return true;
        }
        let queens = self.piece_bitboards[Piece::new(&PieceType::Queen, &self.side_to_move.enemy())];
        if (get_queen_attacks(&square, &self.occupied_squares) & queens).0 != 0 {
            return true;
        }
        let king = self.piece_bitboards[Piece::new(&PieceType::King, &self.side_to_move.enemy())];
        if (KING_ATTACK_MASKS[square] & king).0 != 0 {
            return true;
        }
        false
    }
    fn xray_rook_attacks(&self, square: &usize) -> Bitboard {
        let attacks = get_rook_attacks(square, &self.occupied_squares);
        let blockers = self.side_bitboards[self.side_to_move.enemy()] & attacks;
        attacks ^ get_rook_attacks(square, &(self.occupied_squares ^ blockers))
    }
    fn xray_bishop_attacks(&self, square: &usize) -> Bitboard {
        let attacks = get_bishop_attacks(square, &self.occupied_squares);
        let blockers = self.side_bitboards[self.side_to_move.enemy()] & attacks;
        attacks ^ get_bishop_attacks(square, &(self.occupied_squares ^ blockers))
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
            ..Self::default()
        }
    }
}

#[derive(Clone, Copy)]
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
    fn load_fen_sets_correct_squares() {
        let mut squares = [Option::<Piece>::None; 64];
        for i in 0..8 {
            squares[8 + i] = Some(Piece::new(&PieceType::Pawn, &Side::Black));
            squares[48 + i] = Some(Piece::new(&PieceType::Pawn, &Side::White));
        }
        squares[0] = Some(Piece::new(&PieceType::Rook, &Side::Black));
        squares[1] = Some(Piece::new(&PieceType::Knight, &Side::Black));
        squares[2] = Some(Piece::new(&PieceType::Bishop, &Side::Black));
        squares[3] = Some(Piece::new(&PieceType::Queen, &Side::Black));
        squares[4] = Some(Piece::new(&PieceType::King, &Side::Black));
        squares[5] = Some(Piece::new(&PieceType::Bishop, &Side::Black));
        squares[6] = Some(Piece::new(&PieceType::Knight, &Side::Black));
        squares[7] = Some(Piece::new(&PieceType::Rook, &Side::Black));

        squares[56] = Some(Piece::new(&PieceType::Rook, &Side::White));
        squares[57] = Some(Piece::new(&PieceType::Knight, &Side::White));
        squares[58] = Some(Piece::new(&PieceType::Bishop, &Side::White));
        squares[59] = Some(Piece::new(&PieceType::Queen, &Side::White));
        squares[60] = Some(Piece::new(&PieceType::King, &Side::White));
        squares[61] = Some(Piece::new(&PieceType::Bishop, &Side::White));
        squares[62] = Some(Piece::new(&PieceType::Knight, &Side::White));
        squares[63] = Some(Piece::new(&PieceType::Rook, &Side::White));

        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        assert_eq!(squares, board.squares);
    }
    #[test]
    fn sets_correct_bitboards_from_squares() {
        let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        board.set_bitboards_from_squares();
        let white_pawn_bitboard = board.piece_bitboards[Piece::new(&PieceType::Pawn, &Side::White)];
        assert_eq!(white_pawn_bitboard.0, 0x00FF000000000000)
    }
}
