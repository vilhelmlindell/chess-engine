use super::bitboard::Bitboard;
use super::direction::Direction;
use super::piece::{Piece, PieceType};
use super::piece_move::{Move, MoveType, Square};
use super::utils::flip_rank;
use super::zobrist_hash::{get_zobrist_castling_rights, get_zobrist_en_passant_square, get_zobrist_hash, get_zobrist_side, get_zobrist_squares};
use crate::evaluation::piece_square_tables::{endgame_position_value, midgame_position_value};
use crate::move_generation::attack_tables::*;
use crate::search::transposition_table::*;
use num_enum::UnsafeFromPrimitive;
use pyrrhic_rs::EngineAdapter;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Index, IndexMut};
use std::u64;

const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub const RANK_1: Bitboard = Bitboard(0xFF00000000000000);
pub const RANK_2: Bitboard = Bitboard(0x00FF000000000000);
pub const RANK_3: Bitboard = Bitboard(0x0000FF0000000000);
pub const RANK_4: Bitboard = Bitboard(0x000000FF00000000);
pub const RANK_5: Bitboard = Bitboard(0x00000000FF000000);
pub const RANK_6: Bitboard = Bitboard(0x0000000000FF0000);
pub const RANK_7: Bitboard = Bitboard(0x000000000000FF00);
pub const RANK_8: Bitboard = Bitboard(0x00000000000000FF);
pub const RANKS: [Bitboard; 8] = [RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8];
pub const TOTAL_MATERIAL_STARTPOS: u32 = 16 * PieceType::Pawn.standard_value() + 4 * PieceType::Knight.standard_value() + 4 * PieceType::Bishop.standard_value() + 4 * PieceType::Rook.standard_value() + 2 * PieceType::Queen.standard_value();

const BLACK_QUEENSIDE_START_MASK: Bitboard = Bitboard((1 << 0) | (1 << 4)); // a8 and e8
const BLACK_KINGSIDE_START_MASK: Bitboard = Bitboard((1 << 7) | (1 << 4)); // h8 and e8
const WHITE_QUEENSIDE_START_MASK: Bitboard = Bitboard((1 << 56) | (1 << 60)); // a1 and e1
const WHITE_KINGSIDE_START_MASK: Bitboard = Bitboard((1 << 63) | (1 << 60)); // h1 and e1
const CASTLING_START_SQUARES: Bitboard = Bitboard((1 << 0) | (1 << 4) | (1 << 7) | (1 << 56) | (1 << 60) | (1 << 63));

pub fn square_from_string(square: &str) -> usize {
    let files = "abcdefgh";
    let rank = files.chars().position(|char| square.as_bytes()[0] as char == char).unwrap();
    let file = (8 - (square.as_bytes()[1] as char).to_digit(10).unwrap()) as usize;
    file * 8 + rank
}

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
    pub total_material: u32,
    pub midgame_position_balance: i32,
    pub endgame_position_balance: i32,
    pub transposition_table: TranspositionTable,
    pub zobrist_hash: u64,
    pub opening_move_count: u32,
    pub ply: u32,
    pub can_detect_threefold_repetition: bool,
    pub orthogonal_pinmask: Bitboard,
    pub diagonal_pinmask: Bitboard,
    pub checkmask: Bitboard,
}

impl Board {
    pub fn from_fen(fen: &str) -> Self {
        let mut board = Self::default();
        board.load_fen(fen);
        board.absolute_pinned_squares = board.absolute_pins();
        board.checkmask = board.checkmask();
        let square = board.piece_squares[Piece::new(PieceType::King, board.side)].lsb();
        board.orthogonal_pinmask = get_orthogonal_rays(square);
        board.diagonal_pinmask = get_diagonal_rays(square);
        board
    }
    pub fn fen(&self) -> String {
        let mut fen = "".to_string();
        let piece_types = HashMap::from([(PieceType::Pawn, 'p'), (PieceType::Knight, 'n'), (PieceType::Bishop, 'b'), (PieceType::Rook, 'r'), (PieceType::Queen, 'q'), (PieceType::King, 'k')]);
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
        fen.push(' ');
        fen.push_str(&self.state().halfmove_clock.to_string());
        fen.push_str(&(self.ply / 2).to_string());
        fen
    }
    pub fn start_pos() -> Self {
        let mut board = Self::from_fen(STARTING_FEN);
        board.can_detect_threefold_repetition = true;
        return board;
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
        Option::is_some(&self.squares[mov.to()])
    }

    pub fn load_fen(&mut self, fen: &str) {
        let piece_types = HashMap::from([('p', PieceType::Pawn), ('n', PieceType::Knight), ('b', PieceType::Bishop), ('r', PieceType::Rook), ('q', PieceType::Queen), ('k', PieceType::King)]);
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
            _ => self.state_mut().en_passant_square = Some(square_from_string(fields.get(3).unwrap())),
        }

        self.state_mut().halfmove_clock = fields.get(4).unwrap().parse::<u8>().unwrap();
        self.ply = fields.get(4).unwrap().parse::<u32>().unwrap() * 2 + self.side as u32;

        self.initialize_bitboards();
        for square in 0..64 {
            if let Some(piece) = self.squares[square] {
                self.material_balance += piece.piece_type().centipawns() * piece.side().factor();
                self.midgame_position_balance += midgame_position_value(piece.piece_type(), square, piece.side()) * piece.side().factor();
                self.endgame_position_balance += endgame_position_value(piece.piece_type(), square, piece.side()) * piece.side().factor();
                self.total_material += piece.piece_type().standard_value();
            }
        }
        self.zobrist_hash = get_zobrist_hash(self);
    }
    pub fn make_move(&mut self, mov: Move) {
        let mut state = BoardState::from_state(self.state());

        let castling_rights_bits_before = Self::castling_rights_bits(self.state().castling_rights);
        let move_bitboard = Bitboard::from_square(mov.from()) | Bitboard::from_square(mov.to());

        if move_bitboard & CASTLING_START_SQUARES != 0 {
            if move_bitboard & BLACK_QUEENSIDE_START_MASK != 0 {
                state.castling_rights[Side::Black].queenside = false;
            }
            if move_bitboard & BLACK_KINGSIDE_START_MASK != 0 {
                state.castling_rights[Side::Black].kingside = false;
            }
            if move_bitboard & WHITE_QUEENSIDE_START_MASK != 0 {
                state.castling_rights[Side::White].queenside = false;
            }
            if move_bitboard & WHITE_KINGSIDE_START_MASK != 0 {
                state.castling_rights[Side::White].kingside = false;
            }
        }

        let castling_rights_bits_after = Self::castling_rights_bits(state.castling_rights);
        self.zobrist_hash ^= get_zobrist_castling_rights(castling_rights_bits_before) ^ get_zobrist_castling_rights(castling_rights_bits_after);

        if let Some(captured_piece) = self.squares[mov.to()] {
            self.clear_square(mov.to());
            state.captured_piece = Some(captured_piece);
            state.halfmove_clock = 0;
            state.last_irreversible_ply = self.ply;
        }

        if self.squares[mov.from()].expect("Invalid move: no piece on from square").piece_type() == PieceType::Pawn {
            state.halfmove_clock = 0;
            state.last_irreversible_ply = self.ply;
        }

        if let Some(prev_en_passant_square) = self.state().en_passant_square {
            let prev_file = prev_en_passant_square % 8;
            self.zobrist_hash ^= get_zobrist_en_passant_square(prev_file);
        }

        self.move_piece(mov.from(), mov.to());

        match mov.move_type() {
            MoveType::Normal => {}
            MoveType::KingsideCastle => {
                state.last_irreversible_ply = self.ply;
                let (rook_from, rook_to) = if self.side == Side::White { (63, 61) } else { (7, 5) };
                self.move_piece(rook_from, rook_to);
            }
            MoveType::QueensideCastle => {
                state.last_irreversible_ply = self.ply;
                let (rook_from, rook_to) = if self.side == Side::White { (56, 59) } else { (0, 3) };
                self.move_piece(rook_from, rook_to);
            }
            MoveType::DoublePush => {
                state.en_passant_square = Some((mov.to() as i32 + Direction::down(self.side).value()) as usize);
                let file = state.en_passant_square.unwrap() % 8;
                self.zobrist_hash ^= get_zobrist_en_passant_square(file);
            }
            MoveType::EnPassant => {
                let en_passant_square = self.state().en_passant_square.unwrap();
                let capture_square = (en_passant_square as i32 + Direction::down(self.side).value()) as usize;
                let captured_piece = self.squares[capture_square].unwrap();
                self.clear_square(capture_square);
                state.captured_piece = Some(captured_piece);
            }
            MoveType::RookPromotion | MoveType::QueenPromotion | MoveType::BishopPromotion | MoveType::KnightPromotion => {
                let piece_type = mov.move_type().promotion_piece();
                let piece = Piece::new(piece_type, self.side);
                self.clear_square(mov.to());
                self.set_square(mov.to(), piece);
            }
        }

        self.side = self.side.enemy();
        self.zobrist_hash ^= get_zobrist_side();
        self.absolute_pinned_squares = self.absolute_pins();

        //self.checkmask = self.checkmask();
        //let square = self.piece_squares[Piece::new(PieceType::King, self.side)].lsb();
        //self.orthogonal_pinmask = get_orthogonal_rays(square);
        //self.diagonal_pinmask = get_diagonal_rays(square);

        if self.state().halfmove_clock != 0 {
            state.halfmove_clock += 1;
        }
        self.ply += 1;
        self.states.push(state);
    }
    pub fn unmake_move(&mut self, mov: Move) {
        self.ply -= 1;
        self.side = self.side.enemy();
        self.zobrist_hash ^= get_zobrist_side();

        let castling_rights_bits_before = Self::castling_rights_bits(self.state().castling_rights);

        self.move_piece(mov.to(), mov.from());

        if mov.move_type() != MoveType::EnPassant {
            if let Some(piece) = self.state_mut().captured_piece {
                self.set_square(mov.to(), piece);
            }
        }

        match mov.move_type() {
            MoveType::KingsideCastle => {
                let (rook_from, rook_to) = if self.side == Side::White { (61, 63) } else { (5, 7) };
                self.move_piece(rook_from, rook_to);
            }
            MoveType::QueensideCastle => {
                let (rook_from, rook_to) = if self.side == Side::White { (59, 56) } else { (3, 0) };
                self.move_piece(rook_from, rook_to);
            }
            MoveType::DoublePush => {
                let file = self.state().en_passant_square.unwrap() % 8;
                self.zobrist_hash ^= get_zobrist_en_passant_square(file);
            }
            MoveType::EnPassant => {
                let square = (self.states[self.states.len() - 2].en_passant_square.unwrap() as i32 + Direction::down(self.side).value()) as usize;
                let captured_pawn = Piece::new(PieceType::Pawn, self.side.enemy());
                self.set_square(square, captured_pawn);
            }
            MoveType::RookPromotion | MoveType::QueenPromotion | MoveType::BishopPromotion | MoveType::KnightPromotion => {
                let pawn = Piece::new(PieceType::Pawn, self.side);
                self.clear_square(mov.from());
                self.set_square(mov.from(), pawn);
            }
            _ => {}
        }

        self.absolute_pinned_squares = self.absolute_pins();

        //self.checkmask = self.checkmask();
        //let square = self.piece_squares[Piece::new(PieceType::King, self.side)].lsb();
        //self.orthogonal_pinmask = get_orthogonal_rays(square);
        //self.diagonal_pinmask = get_diagonal_rays(square);
        self.states.pop();

        let castling_rights_bits_after = Self::castling_rights_bits(self.state().castling_rights);

        if let Some(prev_en_passant_square) = self.state().en_passant_square {
            let prev_file = prev_en_passant_square % 8;
            self.zobrist_hash ^= get_zobrist_en_passant_square(prev_file);
        }

        self.zobrist_hash ^= get_zobrist_castling_rights(castling_rights_bits_before) ^ get_zobrist_castling_rights(castling_rights_bits_after);
    }
    pub fn make_null_move(&mut self) {
        let state = BoardState::from_state(self.state());

        let castling_rights_bits_before = Self::castling_rights_bits(self.state().castling_rights);

        let castling_rights_bits_after = Self::castling_rights_bits(state.castling_rights);
        self.zobrist_hash ^= get_zobrist_castling_rights(castling_rights_bits_before) ^ get_zobrist_castling_rights(castling_rights_bits_after);

        if let Some(prev_en_passant_square) = self.state().en_passant_square {
            let prev_file = prev_en_passant_square % 8;
            self.zobrist_hash ^= get_zobrist_en_passant_square(prev_file);
        }

        self.side = self.side.enemy();
        self.zobrist_hash ^= get_zobrist_side();
        self.absolute_pinned_squares = self.absolute_pins();

        self.ply += 1;
        self.states.push(state);
    }

    pub fn unmake_null_move(&mut self) {
        self.ply -= 1;
        self.side = self.side.enemy();
        self.zobrist_hash ^= get_zobrist_side();

        let castling_rights_bits_before = Self::castling_rights_bits(self.state().castling_rights);

        self.absolute_pinned_squares = self.absolute_pins();
        self.states.pop();

        let castling_rights_bits_after = Self::castling_rights_bits(self.state().castling_rights);

        if let Some(prev_en_passant_square) = self.state().en_passant_square {
            let prev_file = prev_en_passant_square % 8;
            self.zobrist_hash ^= get_zobrist_en_passant_square(prev_file);
        }

        self.zobrist_hash ^= get_zobrist_castling_rights(castling_rights_bits_before) ^ get_zobrist_castling_rights(castling_rights_bits_after);
    }
    #[inline(always)]
    pub fn attacked(&self, square: usize) -> bool {
        let pawns = self.piece_squares[Piece::new(PieceType::Pawn, self.side.enemy())];
        let knights = self.piece_squares[Piece::new(PieceType::Knight, self.side.enemy())];
        let queens = self.piece_squares[Piece::new(PieceType::Queen, self.side.enemy())];
        let bishops = self.piece_squares[Piece::new(PieceType::Bishop, self.side.enemy())];
        let rooks = self.piece_squares[Piece::new(PieceType::Rook, self.side.enemy())];
        ((get_pawn_attack(self.side.enemy(), square) & pawns) | (get_knight_attack_mask(square) & knights) | (bishop_attacks(square, self.occupied_squares) & (bishops | queens)) | (rook_attacks(square, self.occupied_squares) & (rooks | queens))) != 0
    }
    #[inline(always)]
    pub fn attackers(&self, square: usize, side: Side) -> Bitboard {
        let mut attackers = Bitboard(0);

        let pawns = self.piece_squares[Piece::new(PieceType::Pawn, side.enemy())];
        attackers |= get_pawn_attack(side.enemy(), square) & pawns;

        let knights = self.piece_squares[Piece::new(PieceType::Knight, side.enemy())];
        attackers |= get_knight_attack_mask(square) & knights;

        let queens = self.piece_squares[Piece::new(PieceType::Queen, side.enemy())];

        let bishops = self.piece_squares[Piece::new(PieceType::Bishop, side.enemy())];
        attackers |= bishop_attacks(square, self.occupied_squares) & (bishops | queens);

        let rooks = self.piece_squares[Piece::new(PieceType::Rook, side.enemy())];
        attackers |= rook_attacks(square, self.occupied_squares) & (rooks | queens);

        attackers
    }
    pub fn in_check(&self) -> bool {
        let king_square = self.piece_squares[Piece::new(PieceType::King, self.side)].lsb();
        return self.attacked(king_square);
    }
    #[inline(always)]
    pub fn king_attacked(&self, from: usize, to: usize) -> bool {
        let pawns = self.piece_squares[Piece::new(PieceType::Pawn, self.side.enemy())];
        let knights = self.piece_squares[Piece::new(PieceType::Knight, self.side.enemy())];
        let queens = self.piece_squares[Piece::new(PieceType::Queen, self.side.enemy())];
        let bishops = self.piece_squares[Piece::new(PieceType::Bishop, self.side.enemy())];
        let occupied = self.occupied_squares ^ Bitboard::from_square(from);
        let rooks = self.piece_squares[Piece::new(PieceType::Rook, self.side.enemy())];

        ((get_pawn_attack(self.side.enemy(), to) & pawns) | (get_knight_attack_mask(to) & knights) | (bishop_attacks(to, occupied) & (bishops | queens)) | (rook_attacks(to, occupied) & (rooks | queens))) != 0
    }
    #[inline(always)]
    pub fn aligned(square1: usize, square2: usize, square3: usize) -> bool {
        get_line_ray(square1, square2) & Bitboard::from_square(square3) != 0
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
        self.midgame_position_balance += midgame_position_value(piece.piece_type(), square, piece.side()) * piece.side().factor();
        self.endgame_position_balance += endgame_position_value(piece.piece_type(), square, piece.side()) * piece.side().factor();
        self.material_balance += piece.piece_type().centipawns() * piece.side().factor();
        self.total_material += piece.piece_type().standard_value();
        self.zobrist_hash ^= get_zobrist_squares(square, piece);
    }

    #[inline(always)]
    fn clear_square(&mut self, square: usize) {
        let piece = self.squares[square].unwrap();
        self.occupied_squares.clear_bit(square);
        self.side_squares[piece.side()].clear_bit(square);
        self.piece_squares[piece].clear_bit(square);
        self.squares[square] = None;
        self.midgame_position_balance -= midgame_position_value(piece.piece_type(), square, piece.side()) * piece.side().factor();
        self.endgame_position_balance -= endgame_position_value(piece.piece_type(), square, piece.side()) * piece.side().factor();
        self.material_balance -= piece.piece_type().centipawns() * piece.side().factor();
        self.total_material -= piece.piece_type().standard_value();
        self.zobrist_hash ^= get_zobrist_squares(square, piece);
    }
    #[inline(always)]
    fn checkmask(&mut self) -> Bitboard {
        let square = self.piece_squares[Piece::new(PieceType::King, self.side)].lsb();
        let mut attackers = Bitboard(0);

        let pawns = self.piece_squares[Piece::new(PieceType::Pawn, self.side.enemy())];
        let knights = self.piece_squares[Piece::new(PieceType::Knight, self.side.enemy())];
        let queens = self.piece_squares[Piece::new(PieceType::Queen, self.side.enemy())];
        let bishops = self.piece_squares[Piece::new(PieceType::Bishop, self.side.enemy())];
        let rooks = self.piece_squares[Piece::new(PieceType::Rook, self.side.enemy())];

        attackers |= get_pawn_attack(self.side.enemy(), square) & pawns;
        attackers |= get_knight_attack_mask(square) & knights;
        attackers |= bishop_attacks(square, self.occupied_squares) & (bishops | queens);
        attackers |= rook_attacks(square, self.occupied_squares) & (rooks | queens);

        let attacker_square = attackers.lsb();
        let attacker_count = attackers.count_ones();

        if attacker_count > 2 {
            return Bitboard(0);
        } else if attacker_count == 1 {
            return get_checkmask_between(square, attacker_square);
        } else {
            return Bitboard(u64::MAX);
        }
    }
    #[inline(always)]
    fn absolute_pins(&self) -> Bitboard {
        let king_square = self.piece_squares[Piece::new(PieceType::King, self.side)].lsb();

        let mut pinned_squares = Bitboard(0);
        let mut pinners = self.xray_rook_attacks(king_square, self.friendly_squares()) & (self.piece_squares[Piece::new(PieceType::Rook, self.side.enemy())] | self.piece_squares[Piece::new(PieceType::Queen, self.side.enemy())]);

        while pinners != 0 {
            let pinner_square = pinners.pop_lsb();
            pinned_squares |= get_between_ray(king_square, pinner_square) & self.friendly_squares();
        }

        pinners = self.xray_bishop_attacks(king_square, self.friendly_squares()) & (self.piece_squares[Piece::new(PieceType::Bishop, self.side.enemy())] | self.piece_squares[Piece::new(PieceType::Queen, self.side.enemy())]);

        while pinners != 0 {
            let pinner_square = pinners.pop_lsb();
            pinned_squares |= get_between_ray(king_square, pinner_square) & self.friendly_squares();
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
    fn xray_bishop_attacks(&self, square: usize, blockers: Bitboard) -> Bitboard {
        let attacks = bishop_attacks(square, self.occupied_squares);
        let attacked_blockers = blockers & attacks;
        attacks ^ bishop_attacks(square, self.occupied_squares ^ attacked_blockers)
    }
    #[inline(always)]
    fn castling_rights_bits(castling_rights: [CastlingRights; 2]) -> usize {
        (castling_rights[Side::White].kingside as usize) << 3 | (castling_rights[Side::White].queenside as usize) << 2 | (castling_rights[Side::Black].kingside as usize) << 1 | castling_rights[Side::Black].queenside as usize
    }
}

// NOTE: pyrrheic_rs uses A1=0, while my engine uses A8=0. This leads to me needing to convert
// between these two representations when calling probing tablebases. Possible fix is to change my
// engine to A1=0 but that's a pain in the ass
impl EngineAdapter for Board {
    fn pawn_attacks(color: pyrrhic_rs::Color, square: u64) -> u64 {
        if color == pyrrhic_rs::Color::White {
            get_pawn_attack(Side::Black, flip_rank(square as usize)).swap_bytes()
        } else {
            get_pawn_attack(Side::White, flip_rank(square as usize)).swap_bytes()
        }
    }

    fn knight_attacks(square: u64) -> u64 {
        get_knight_attack_mask(flip_rank(square as usize)).swap_bytes()
    }

    fn bishop_attacks(square: u64, occupied: u64) -> u64 {
        bishop_attacks(flip_rank(square as usize), Bitboard(occupied.swap_bytes())).swap_bytes()
    }

    fn rook_attacks(square: u64, occupied: u64) -> u64 {
        rook_attacks(flip_rank(square as usize), Bitboard(occupied.swap_bytes())).swap_bytes()
    }

    fn queen_attacks(square: u64, occupied: u64) -> u64 {
        queen_attacks(flip_rank(square as usize), Bitboard(occupied.swap_bytes())).swap_bytes()
    }

    fn king_attacks(square: u64) -> u64 {
        get_king_attack_mask(flip_rank(square as usize)).swap_bytes()
    }
}

impl Default for Board {
    fn default() -> Self {
        Self {
            squares: [None; 64],
            side: Side::White,
            occupied_squares: Bitboard(0),
            side_squares: [Bitboard(0); 2],
            attacked_squares: [Bitboard(0); 2],
            piece_squares: [Bitboard(0); 12],
            absolute_pinned_squares: Bitboard(0),
            states: vec![BoardState::default()],
            material_balance: 0,
            midgame_position_balance: 0,
            endgame_position_balance: 0,
            total_material: 0,
            transposition_table: TranspositionTable::default(),
            zobrist_hash: 0,
            opening_move_count: 0,
            ply: 0,
            can_detect_threefold_repetition: false,
            orthogonal_pinmask: Bitboard(0),
            diagonal_pinmask: Bitboard(0),
            checkmask: Bitboard(0),
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece_chars = HashMap::from([(PieceType::Pawn, 'p'), (PieceType::Knight, 'n'), (PieceType::Bishop, 'b'), (PieceType::Rook, 'r'), (PieceType::Queen, 'q'), (PieceType::King, 'k')]);
        for rank in 0..8 {
            write!(f, "{}", 8 - rank).unwrap();
            for file in 0..8 {
                write!(f, " ").unwrap();
                match self.squares[rank * 8 + file] {
                    Some(piece) => {
                        let piece_char = piece_chars.get(&piece.piece_type()).unwrap();
                        match piece.side() {
                            Side::White => write!(f, "{}", piece_char.to_ascii_uppercase()).unwrap(),
                            Side::Black => write!(f, "{}", piece_char).unwrap(),
                        }
                    }
                    None => write!(f, ".").unwrap(),
                }
            }
            writeln!(f).unwrap();
        }
        write!(f, " ").unwrap();
        for file in 'a'..='h' {
            write!(f, " ").unwrap();
            write!(f, "{}", file).unwrap();
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, UnsafeFromPrimitive)]
#[repr(u8)]
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
    pub halfmove_clock: u8,
    pub last_irreversible_ply: u32,
    pub zobrist_hash: u64,

    // Recalculated
    pub castling_rights: [CastlingRights; 2],
    pub en_passant_square: Option<Square>,
    pub captured_piece: Option<Piece>,
}

impl BoardState {
    pub fn from_state(state: &Self) -> Self {
        Self {
            halfmove_clock: state.halfmove_clock,
            castling_rights: state.castling_rights,
            last_irreversible_ply: state.last_irreversible_ply,

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
            last_irreversible_ply: 0,
            zobrist_hash: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{board, move_generation::generate_moves};

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

        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1");
        assert_eq!(squares, board.squares);
    }
    #[test]
    fn sets_correct_bitboards_from_squares() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1");
        let white_pawn_bitboard = board.piece_squares[Piece::new(PieceType::Pawn, Side::White)];
        assert_eq!(white_pawn_bitboard.0, 0x00FF000000000000)
    }
    #[test]
    fn test_aligned() {
        assert!(Board::aligned(28, 44, 60));
        assert!(!Board::aligned(43, 44, 60));
    }
    #[test]
    fn make_unmake() {
        //let original = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/1B1PN3/1p2P3/2N2Q1p/PPPB1PPP/R3K2R b KQkq - 1 1");
        //let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/1B1PN3/1p2P3/2N2Q1p/PPPB1PPP/R3K2R b KQkq - 1 1");
        let original = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let zobrist_before = board.zobrist_hash;
        for mov in generate_moves(&board) {
            board.make_move(mov);
            board.unmake_move(mov);
            board.make_null_move();
            board.unmake_null_move();
            println!();
            println!("{original}");
            println!("{board}");
            println!("{mov}");
            assert_eq!(zobrist_before, board.zobrist_hash);

            for square in 0..64 {
                assert!(original.squares[square] == board.squares[square]);
                assert!(original.absolute_pins() == board.absolute_pins());
            }
        }
    }
    #[test]
    fn engine_adapter() {
        let board = Board::from_fen("4k3/8/3QK3/8/8/8/8/8 w - - 0 1");
        println!("{}", Bitboard(Board::queen_attacks(board.piece_squares[Piece::WhiteQueen].lsb() as u64, board.occupied_squares.0)));
        println!("{}", board.piece_squares[Piece::WhiteQueen]);
    }
}
