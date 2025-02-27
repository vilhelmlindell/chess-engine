use crate::board::bitboard::Bitboard;
use crate::board::direction::{self, Direction};
use crate::board::piece_move::Square;
use crate::board::Side;
use crate::move_generation::magic_numbers::*;
use std::cmp::min;
use std::num::Wrapping;
use std::sync::LazyLock;
use ctor::ctor;

static mut SQUARES_TO_EDGE: [[u32; 8]; 64] = [[0; 8]; 64];
static mut ATTACK_RAYS: [[Bitboard; 8]; 64] = [[Bitboard(0); 8]; 64];
static mut KNIGHT_ATTACK_MASKS: [Bitboard; 64] = [Bitboard(0); 64];
static mut KING_ATTACK_MASKS: [Bitboard; 64] = [Bitboard(0); 64];
static mut BISHOP_ATTACK_MASKS: [Bitboard; 64] = [Bitboard(0); 64];
static mut ROOK_ATTACK_MASKS: [Bitboard; 64] = [Bitboard(0); 64];
static mut BISHOP_ATTACKS: [[Bitboard; 512]; 64] = [[Bitboard(0); 512]; 64];
static mut ROOK_ATTACKS: [[Bitboard; 4096]; 64] = [[Bitboard(0); 4096]; 64];
static mut PAWN_ATTACKS: [[Bitboard; 64]; 2] = [[Bitboard(0); 64]; 2];
static mut BETWEEN_RAYS: [[Bitboard; 64]; 64] = [[Bitboard(0); 64]; 64];
static mut LINE_RAYS: [[Bitboard; 64]; 64] = [[Bitboard(0); 64]; 64];
static mut CHECKMASK_BETWEEN: [[Bitboard; 64]; 64] = [[Bitboard(0); 64]; 64];
static mut ORTHOGONAL_RAYS: [Bitboard; 64] = [Bitboard(0); 64];
static mut DIAGONAL_RAYS: [Bitboard; 64] = [Bitboard(0); 64];

#[ctor]
pub fn initialize_tables() {
    unsafe {
        SQUARES_TO_EDGE = precompute_squares_to_edge();
        ATTACK_RAYS = precompute_attack_rays();
        KNIGHT_ATTACK_MASKS = precompute_knight_attack_masks();
        KING_ATTACK_MASKS = precompute_king_attack_masks();
        BISHOP_ATTACK_MASKS = precompute_bishop_attack_masks();
        ROOK_ATTACK_MASKS = precompute_rook_attack_masks();
        BISHOP_ATTACKS = precompute_bishop_magic_bitboards();
        ROOK_ATTACKS = precompute_rook_magic_bitboards();
        PAWN_ATTACKS = precompute_pawn_attacks();
        BETWEEN_RAYS = precompute_between_rays();
        LINE_RAYS = precompute_line_rays();
        CHECKMASK_BETWEEN = precompute_checkmask_between();
    }
}

#[inline(always)]
pub fn get_squares_to_edge(square: usize, direction: Direction) -> u32 {
    unsafe {  
        SQUARES_TO_EDGE[square][direction as usize]
    }
}

#[inline(always)]
pub fn get_attack_ray(square: usize, direction: Direction) -> Bitboard {
    unsafe {
        ATTACK_RAYS[square][direction as usize]
    }
}

#[inline(always)]
pub fn get_knight_attack_mask(square: usize) -> Bitboard {
    unsafe {
        KNIGHT_ATTACK_MASKS[square]
    }
}

#[inline(always)]
pub fn get_king_attack_mask(square: usize) -> Bitboard {
    unsafe {
        KING_ATTACK_MASKS[square]
    }
}

#[inline(always)]
pub fn get_bishop_attack_mask(square: usize) -> Bitboard {
    unsafe {
        BISHOP_ATTACK_MASKS[square]
    }
}

#[inline(always)]
pub fn get_rook_attack_mask(square: usize) -> Bitboard {
    unsafe {
        ROOK_ATTACK_MASKS[square]
    }
}

#[inline(always)]
pub fn get_bishop_attack(square: usize, index: usize) -> Bitboard {
    unsafe {
        BISHOP_ATTACKS[square][index]
    }
}

#[inline(always)]
pub fn get_rook_attack(square: usize, index: usize) -> Bitboard {
    unsafe {
        ROOK_ATTACKS[square][index]
    }
}

#[inline(always)]
pub fn get_pawn_attack(side: Side, square: usize) -> Bitboard {
    unsafe {
        PAWN_ATTACKS[side as usize][square]
    }
}

#[inline(always)]
pub fn get_between_ray(square1: usize, square2: usize) -> Bitboard {
    unsafe {
        BETWEEN_RAYS[square1][square2]
    }
}

#[inline(always)]
pub fn get_line_ray(square1: usize, square2: usize) -> Bitboard {
    unsafe {
        LINE_RAYS[square1][square2]
    }
}

#[inline(always)]
pub fn get_checkmask_between(square1: usize, square2: usize) -> Bitboard {
    unsafe {
        CHECKMASK_BETWEEN[square1][square2]
    }
}
#[inline(always)]
pub fn get_orthogonal_rays(square: Square) -> Bitboard {
    unsafe {
        ORTHOGONAL_RAYS[square]
    }
}
#[inline(always)]
pub fn get_diagonal_rays(square: Square) -> Bitboard {
    unsafe {
        DIAGONAL_RAYS[square]
    }
}

#[inline]
pub fn bishop_attacks(square: usize, blockers: Bitboard) -> Bitboard {
    let mut index = Wrapping(blockers.0);
    index &= get_bishop_attack_mask(square).0;
    index *= BISHOP_MAGIC_NUMBERS[square];
    index >>= 64 - BISHOP_SHIFT_AMOUNT[square] as usize;
    get_bishop_attack(square, index.0 as usize)
}
#[inline]
pub fn rook_attacks(square: usize, blockers: Bitboard) -> Bitboard {
    let mut index = Wrapping(blockers.0);
    index &= get_rook_attack_mask(square).0;
    index *= ROOK_MAGIC_NUMBERS[square];
    index >>= 64 - ROOK_SHIFT_AMOUNT[square] as usize;
    get_rook_attack(square, index.0 as usize)
}
#[inline]
pub fn queen_attacks(square: usize, blockers: Bitboard) -> Bitboard {
    bishop_attacks(square, blockers) | rook_attacks(square, blockers)
}

fn precompute_squares_to_edge() -> [[u32; 8]; 64] {
    let mut squares_to_edge = [[0; 8]; 64];
    for file in 0..8 {
        for rank in 0..8 {
            let north = rank as u32;
            let south = (7 - rank) as u32;
            let west = file as u32;
            let east = (7 - file) as u32;

            let square = rank * 8 + file;

            squares_to_edge[square][Direction::North] = north;
            squares_to_edge[square][Direction::South] = south;
            squares_to_edge[square][Direction::West] = west;
            squares_to_edge[square][Direction::East] = east;
            squares_to_edge[square][Direction::NorthWest] = min(north, west);
            squares_to_edge[square][Direction::NorthEast] = min(north, east);
            squares_to_edge[square][Direction::SouthWest] = min(south, west);
            squares_to_edge[square][Direction::SouthEast] = min(south, east);
        }
    }
    squares_to_edge
}
fn precompute_attack_rays() -> [[Bitboard; 8]; 64] {
    let mut attack_rays = [[Bitboard(0); 8]; 64];
    for square in 0..64 {
        let mut square_attack_rays = [Bitboard(0); 8];
        for direction in Direction::all() {
            let mut attack_ray = Bitboard(0);
            for squares_to_edge in 1..get_squares_to_edge(square, direction) + 1 {
                let end_square = square as i32 + direction.value() * squares_to_edge as i32;
                attack_ray.set_bit(end_square as usize);
            }
            square_attack_rays[direction] = attack_ray;
        }
        attack_rays[square] = square_attack_rays;
    }
    attack_rays
}
fn precompute_between_rays() -> [[Bitboard; 64]; 64] {
    let mut between_rays = [[Bitboard(0); 64]; 64];
    for square in 0..64 {
        let mut square_between_rays = [Bitboard(0); 64];
        for direction in Direction::all() {
            for squares_to_edge in 1..get_squares_to_edge(square, direction) + 1 {
                let end_square = (square as i32 + direction.value() * squares_to_edge as i32) as usize;
                square_between_rays[end_square] = get_attack_ray(square, direction) ^ get_attack_ray(end_square, direction);
            }
        }
        between_rays[square] = square_between_rays;
    }
    between_rays
}
fn precompute_line_rays() -> [[Bitboard; 64]; 64] {
    let mut line_rays = [[Bitboard(0); 64]; 64];
    for square in 0..64 {
        let mut square_line_rays = [Bitboard(0); 64];
        for direction in Direction::all() {
            for squares_to_edge in 1..=get_squares_to_edge(square, direction) {
                let end_square = (square as i32 + direction.value() * squares_to_edge as i32) as usize;
                square_line_rays[end_square] = get_attack_ray(square, direction) | get_attack_ray(square, direction.opposite()) | Bitboard::from_square(square);
            }
        }
        line_rays[square] = square_line_rays;
    }
    line_rays
}
fn precompute_knight_attack_masks() -> [Bitboard; 64] {
    let mut knight_attack_masks = [Bitboard(0); 64];
    let directions = [
        Direction::North.value() * 2 + Direction::West.value(),
        Direction::North.value() * 2 + Direction::East.value(),
        Direction::South.value() * 2 + Direction::West.value(),
        Direction::South.value() * 2 + Direction::East.value(),
        Direction::West.value() * 2 + Direction::North.value(),
        Direction::West.value() * 2 + Direction::South.value(),
        Direction::East.value() * 2 + Direction::North.value(),
        Direction::East.value() * 2 + Direction::South.value(),
    ];
    for square in 0..64 {
        let mut knight_attack_bitboard = Bitboard(0);
        for direction in directions {
            let end_square = square + direction;
            if (0..64).contains(&end_square) && i32::abs(square % 8 - end_square % 8) <= 2 {
                knight_attack_bitboard.set_bit(end_square as usize);
            }
        }
        knight_attack_masks[square as usize] = knight_attack_bitboard;
    }
    knight_attack_masks
}
fn precompute_king_attack_masks() -> [Bitboard; 64] {
    let mut king_attack_masks = [Bitboard(0); 64];
    for square in 0..64 {
        let mut king_attack_bitboard = Bitboard(0);
        for direction in Direction::all() {
            let end_square = square + direction.value();
            if (0..64).contains(&end_square) && i32::abs(square % 8 - end_square % 8) <= 2 {
                king_attack_bitboard.set_bit(end_square as usize);
            }
        }
        king_attack_masks[square as usize] = king_attack_bitboard;
    }
    king_attack_masks
}
fn precompute_rook_attack_masks() -> [Bitboard; 64] {
    let mut attack_masks = [Bitboard(0); 64];
    for square in 0..64 {
        let mut attack_mask = Bitboard(0);
        for direction in Direction::orthogonal() {
            for squares_to_edge in 1..get_squares_to_edge(square, direction) {
                let end_square = square as i32 + direction.value() * squares_to_edge as i32;
                attack_mask.set_bit(end_square as usize);
            }
        }
        attack_masks[square] = attack_mask;
    }
    attack_masks
}
fn precompute_bishop_attack_masks() -> [Bitboard; 64] {
    let mut attack_masks = [Bitboard(0); 64];
    for square in 0..64 {
        let mut attack_mask = Bitboard(0);
        for direction in Direction::diagonal() {
            for squares_to_edge in 1..get_squares_to_edge(square, direction) {
                let end_square = square as i32 + direction.value() * squares_to_edge as i32;
                attack_mask.set_bit(end_square as usize);
            }
        }
        attack_masks[square] = attack_mask;
    }
    attack_masks
}
fn precompute_pawn_attacks() -> [[Bitboard; 64]; 2] {
    let mut pawn_attacks = [[Bitboard(0); 64]; 2];
    for square in 0..64 {
        let bitboard = Bitboard(1 << square);
        pawn_attacks[Side::White][square] = bitboard.south_west() | bitboard.south_east();
        pawn_attacks[Side::Black][square] = bitboard.north_west() | bitboard.north_east();
    }
    pawn_attacks
}
fn precompute_checkmask_between() -> [[Bitboard; 64]; 64] {
    let mut between_rays = [[Bitboard(0); 64]; 64];
    for square in 0..64 {
        let mut square_between_rays = [Bitboard(0); 64];
        for direction in Direction::all() {
            for squares_to_edge in 1..get_squares_to_edge(square, direction) + 1 {
                let end_square = (square as i32 + direction.value() * squares_to_edge as i32) as usize;
                square_between_rays[end_square] = get_attack_ray(square, direction) ^ get_attack_ray(end_square, direction);
            }
        }
        for square1 in 0..64 {
            if get_between_ray(square, square1) == 0 {
                square_between_rays[square1] = Bitboard::from_square(square1);
            }
        }
        between_rays[square] = square_between_rays;
    }
    between_rays
}
pub fn get_bishop_attacks_classical(square: usize, blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard(0);

    attacks |= get_attack_ray(square, Direction::NorthWest);
    if get_attack_ray(square, Direction::NorthWest) & blockers != 0 {
        let blocker_index = (get_attack_ray(square, Direction::NorthWest) & blockers).msb();
        attacks &= !get_attack_ray(blocker_index, Direction::NorthWest);
    }

    attacks |= get_attack_ray(square, Direction::NorthEast);
    if get_attack_ray(square, Direction::NorthEast) & blockers != 0 {
        let blocker_index = (get_attack_ray(square, Direction::NorthEast) & blockers).msb();
        attacks &= !get_attack_ray(blocker_index, Direction::NorthEast);
    }

    attacks |= get_attack_ray(square, Direction::SouthWest);
    if get_attack_ray(square, Direction::SouthWest) & blockers != 0 {
        let blocker_index = (get_attack_ray(square, Direction::SouthWest) & blockers).lsb();
        attacks &= !get_attack_ray(blocker_index, Direction::SouthWest);
    }

    attacks |= get_attack_ray(square, Direction::SouthEast);
    if get_attack_ray(square, Direction::SouthEast) & blockers != 0 {
        let blocker_index = (get_attack_ray(square, Direction::SouthEast) & blockers).lsb();
        attacks &= !get_attack_ray(blocker_index, Direction::SouthEast);
    }

    attacks
}
pub fn get_rook_attacks_classical(square: usize, blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard(0);

    attacks |= get_attack_ray(square, Direction::North);
    if get_attack_ray(square, Direction::North) & blockers != 0 {
        let blocker_index = (get_attack_ray(square, Direction::North) & blockers).msb();
        attacks &= !get_attack_ray(blocker_index, Direction::North);
    }

    attacks |= get_attack_ray(square, Direction::West);
    if get_attack_ray(square, Direction::West) & blockers != 0 {
        let blocker_index = (get_attack_ray(square, Direction::West) & blockers).msb();
        attacks &= !get_attack_ray(blocker_index, Direction::West);
    }

    attacks |= get_attack_ray(square, Direction::South);
    if get_attack_ray(square, Direction::South) & blockers != 0 {
        let blocker_index = (get_attack_ray(square, Direction::South) & blockers).lsb();
        attacks &= !get_attack_ray(blocker_index, Direction::South);
    }

    attacks |= get_attack_ray(square, Direction::East);
    if get_attack_ray(square, Direction::East) & blockers != 0 {
        let blocker_index = (get_attack_ray(square, Direction::East) & blockers).lsb();
        attacks &= !get_attack_ray(blocker_index, Direction::East);
    }

    attacks
}
fn precompute_bishop_magic_bitboards() -> [[Bitboard; 512]; 64] {
    let mut bishop_attacks = [[Bitboard(0); 512]; 64];
    for square in 0..64 {
        for blocker_combination in 0..(1 << get_bishop_attack_mask(square).count_ones() as u64) + 1 {
            let mut attack_mask = get_bishop_attack_mask(square);
            let mut blockers = Bitboard(0);

            let mut blocker_count = 0;
            while attack_mask.0 != 0 {
                let blocker_index = attack_mask.pop_lsb();
                if (blocker_combination >> blocker_count) & 1 == 1 {
                    blockers.set_bit(blocker_index);
                }
                blocker_count += 1;
            }

            let square_bishop_attacks = get_bishop_attacks_classical(square, blockers);
            let mut index = Wrapping(blockers.0);
            index *= BISHOP_MAGIC_NUMBERS[square];
            index >>= 64 - BISHOP_SHIFT_AMOUNT[square] as usize;
            bishop_attacks[square][index.0 as usize] = square_bishop_attacks;
        }
    }
    bishop_attacks
}
fn precompute_rook_magic_bitboards() -> [[Bitboard; 4096]; 64] {
    let mut rook_attacks = [[Bitboard(0); 4096]; 64];
    let max_blocker_count = get_rook_attack_mask(0).count_ones() as u64;
    for square in 0..64 {
        for blocker_combination in 0..(1 << max_blocker_count) + 1 {
            let mut attack_mask = get_rook_attack_mask(square);
            let mut blockers = Bitboard(0);

            let mut blocker_count = 0;
            while attack_mask.0 != 0 {
                let blocker_index = attack_mask.pop_lsb();
                if (blocker_combination >> blocker_count) & 1 == 1 {
                    blockers.set_bit(blocker_index);
                }
                blocker_count += 1;
            }

            let square_rook_attacks = get_rook_attacks_classical(square, blockers);
            let mut index = Wrapping(blockers.0);
            index *= ROOK_MAGIC_NUMBERS[square];
            index >>= 64 - ROOK_SHIFT_AMOUNT[square] as usize;
            rook_attacks[square][index.0 as usize] = square_rook_attacks;
        }
    }
    rook_attacks
}
fn precompute_orthogonal_rays() -> [Bitboard; 64] {
    let bitboards = [Bitboard(0); 64];
    for square in 0..64 {
        let mut bitboard = Bitboard(0);
        for direction in Direction::orthogonal() {
            bitboard |= get_attack_ray(square, direction);
        }
    }
    bitboards
}
fn precompute_diagonal_rays() -> [Bitboard; 64] {
    let bitboards = [Bitboard(0); 64];
    for square in 0..64 {
        let mut bitboard = Bitboard(0);
        for direction in Direction::diagonal() {
            bitboard |= get_attack_ray(square, direction);
        }
    }
    bitboards
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_of_squares_to_edge() {
        //assert_eq!(SQUARES_TO_EDGE[4][Direction::East], 3);
        //assert_eq!(SQUARES_TO_EDGE[27][Direction::North], 3);
        //assert_eq!(SQUARES_TO_EDGE[47][Direction::South], 2);
        //assert_eq!(SQUARES_TO_EDGE[21][Direction::West], 5);
    }
    #[test]
    fn test_rook_magic_bitboards_indexing() {
        let square = 29;
        let blockers = Bitboard(1 << 37) | Bitboard(1 << 13) | Bitboard(1 << 21);
        assert_eq!(rook_attacks(square, blockers), get_rook_attacks_classical(square, blockers));
    }
}
