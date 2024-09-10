use crate::board::bitboard::Bitboard;
use crate::board::direction::Direction;
use crate::board::Side;
use crate::move_generation::magic_numbers::*;
use once_cell::sync::Lazy;
use std::cmp::min;
use std::fmt::Debug;
use std::num::Wrapping;

pub static SQUARES_TO_EDGE: Lazy<[[u32; 8]; 64]> = Lazy::new(precompute_squares_to_edge);
pub static ATTACK_RAYS: Lazy<[[Bitboard; 8]; 64]> = Lazy::new(precompute_attack_rays);
pub static KNIGHT_ATTACK_MASKS: Lazy<[Bitboard; 64]> = Lazy::new(precompute_knight_attack_masks);
pub static KING_ATTACK_MASKS: Lazy<[Bitboard; 64]> = Lazy::new(precompute_king_attack_masks);
pub static BISHOP_ATTACK_MASKS: Lazy<[Bitboard; 64]> = Lazy::new(precompute_bishop_attack_mask);
pub static ROOK_ATTACK_MASKS: Lazy<[Bitboard; 64]> = Lazy::new(precompute_rook_attack_mask);
pub static BISHOP_ATTACKS: Lazy<Box<[[Bitboard; 512]]>> = Lazy::new(precompute_bishop_magic_bitboards);
pub static ROOK_ATTACKS: Lazy<Box<[[Bitboard; 4096]]>> = Lazy::new(precompute_rook_magic_bitboards);
pub static PAWN_ATTACKS: Lazy<[[Bitboard; 64]; 2]> = Lazy::new(precompute_pawn_attacks);
pub static BETWEEN_RAYS: Lazy<[[Bitboard; 64]; 64]> = Lazy::new(precompute_between_rays);
pub static LINE_RAYS: Lazy<[[Bitboard; 64]; 64]> = Lazy::new(precompute_line_rays);

pub fn bishop_attacks(square: usize, blockers: Bitboard) -> Bitboard {
    let mut index = Wrapping(blockers.0);
    index &= BISHOP_ATTACK_MASKS[square].0;
    index *= BISHOP_MAGIC_NUMBERS[square];
    index >>= 64 - BISHOP_SHIFT_AMOUNT[square] as usize;
    BISHOP_ATTACKS[square][index.0 as usize]
}
pub fn rook_attacks(square: usize, blockers: Bitboard) -> Bitboard {
    let mut index = Wrapping(blockers.0);
    index &= ROOK_ATTACK_MASKS[square].0;
    index *= ROOK_MAGIC_NUMBERS[square];
    index >>= 64 - ROOK_SHIFT_AMOUNT[square] as usize;
    ROOK_ATTACKS[square][index.0 as usize]
}
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
            for squares_to_edge in 1..SQUARES_TO_EDGE[square][direction] + 1 {
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
            for squares_to_edge in 1..SQUARES_TO_EDGE[square][direction] + 1 {
                let end_square = (square as i32 + direction.value() * squares_to_edge as i32) as usize;
                square_between_rays[end_square] = ATTACK_RAYS[square][direction] ^ ATTACK_RAYS[end_square][direction];
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
            for squares_to_edge in 1..=SQUARES_TO_EDGE[square][direction] {
                let end_square = (square as i32 + direction.value() * squares_to_edge as i32) as usize;
                square_line_rays[end_square] = ATTACK_RAYS[square][direction] | ATTACK_RAYS[square][direction.opposite()] | Bitboard::from_square(square);
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
fn precompute_rook_attack_mask() -> [Bitboard; 64] {
    let mut attack_masks = [Bitboard(0); 64];
    for square in 0..64 {
        let mut attack_mask = Bitboard(0);
        for direction in Direction::orthagonal() {
            for squares_to_edge in 1..SQUARES_TO_EDGE[square][direction] {
                let end_square = square as i32 + direction.value() * squares_to_edge as i32;
                attack_mask.set_bit(end_square as usize);
            }
        }
        attack_masks[square] = attack_mask;
    }
    attack_masks
}
fn precompute_bishop_attack_mask() -> [Bitboard; 64] {
    let mut attack_masks = [Bitboard(0); 64];
    for square in 0..64 {
        let mut attack_mask = Bitboard(0);
        for direction in Direction::diagonal() {
            for squares_to_edge in 1..SQUARES_TO_EDGE[square][direction] {
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
pub fn get_bishop_attacks_classical(square: usize, blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard(0);

    attacks |= ATTACK_RAYS[square][Direction::NorthWest];
    if ATTACK_RAYS[square][Direction::NorthWest] & blockers != 0 {
        let blocker_index = (ATTACK_RAYS[square][Direction::NorthWest] & blockers).msb();
        attacks &= !ATTACK_RAYS[blocker_index][Direction::NorthWest];
    }

    attacks |= ATTACK_RAYS[square][Direction::NorthEast];
    if ATTACK_RAYS[square][Direction::NorthEast] & blockers != 0 {
        let blocker_index = (ATTACK_RAYS[square][Direction::NorthEast] & blockers).msb();
        attacks &= !ATTACK_RAYS[blocker_index][Direction::NorthEast];
    }

    attacks |= ATTACK_RAYS[square][Direction::SouthWest];
    if ATTACK_RAYS[square][Direction::SouthWest] & blockers != 0 {
        let blocker_index = (ATTACK_RAYS[square][Direction::SouthWest] & blockers).lsb();
        attacks &= !ATTACK_RAYS[blocker_index][Direction::SouthWest];
    }

    attacks |= ATTACK_RAYS[square][Direction::SouthEast];
    if ATTACK_RAYS[square][Direction::SouthEast] & blockers != 0 {
        let blocker_index = (ATTACK_RAYS[square][Direction::SouthEast] & blockers).lsb();
        attacks &= !ATTACK_RAYS[blocker_index][Direction::SouthEast];
    }

    attacks
}
pub fn get_rook_attacks_classical(square: usize, blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard(0);

    attacks |= ATTACK_RAYS[square][Direction::North];
    if ATTACK_RAYS[square][Direction::North] & blockers != 0 {
        let blocker_index = (ATTACK_RAYS[square][Direction::North] & blockers).msb();
        attacks &= !ATTACK_RAYS[blocker_index][Direction::North];
    }

    attacks |= ATTACK_RAYS[square][Direction::West];
    if ATTACK_RAYS[square][Direction::West] & blockers != 0 {
        let blocker_index = (ATTACK_RAYS[square][Direction::West] & blockers).msb();
        attacks &= !ATTACK_RAYS[blocker_index][Direction::West];
    }

    attacks |= ATTACK_RAYS[square][Direction::South];
    if ATTACK_RAYS[square][Direction::South] & blockers != 0 {
        let blocker_index = (ATTACK_RAYS[square][Direction::South] & blockers).lsb();
        attacks &= !ATTACK_RAYS[blocker_index][Direction::South];
    }

    attacks |= ATTACK_RAYS[square][Direction::East];
    if ATTACK_RAYS[square][Direction::East] & blockers != 0 {
        let blocker_index = (ATTACK_RAYS[square][Direction::East] & blockers).lsb();
        attacks &= !ATTACK_RAYS[blocker_index][Direction::East];
    }

    attacks
}
fn precompute_bishop_magic_bitboards() -> Box<[[Bitboard; 512]]> {
    let mut bishop_attacks = vec![[Bitboard(0); 512]; 64].into_boxed_slice();
    for square in 0..64 {
        for blocker_combination in 0..(1 << BISHOP_ATTACK_MASKS[square].count_ones() as u64) + 1 {
            let mut attack_mask = BISHOP_ATTACK_MASKS[square];
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
fn precompute_rook_magic_bitboards() -> Box<[[Bitboard; 4096]]> {
    let mut rook_attacks = vec![[Bitboard(0); 4096]; 64].into_boxed_slice();
    let max_blocker_count = ROOK_ATTACK_MASKS[0].count_ones() as u64;
    for square in 0..64 {
        for blocker_combination in 0..(1 << max_blocker_count) + 1 {
            let mut attack_mask = ROOK_ATTACK_MASKS[square];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_of_squares_to_edge() {
        assert_eq!(SQUARES_TO_EDGE[4][Direction::East], 3);
        assert_eq!(SQUARES_TO_EDGE[27][Direction::North], 3);
        assert_eq!(SQUARES_TO_EDGE[47][Direction::South], 2);
        assert_eq!(SQUARES_TO_EDGE[21][Direction::West], 5);
    }
    #[test]
    fn test_rook_magic_bitboards_indexing() {
        let square = 29;
        let blockers = Bitboard(1 << 37) | Bitboard(1 << 13) | Bitboard(1 << 21);
        assert_eq!(rook_attacks(square, blockers), get_rook_attacks_classical(square, blockers));
    }
}
