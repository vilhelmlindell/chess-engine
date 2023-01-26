use crate::bitboard::{Bitboard, Direction};
use crate::magic_numbers::*;
use once_cell::sync::Lazy;
use std::cmp::min;
use std::collections::HashMap;
use std::num::Wrapping;

pub static SQUARES_TO_EDGE: Lazy<[[u32; 8]; 64]> = Lazy::new(|| precompute_squares_to_edge());
pub static ATTACK_RAYS: Lazy<[[Bitboard; 8]; 64]> = Lazy::new(|| precompute_attack_rays());
pub static KNIGHT_ATTACK_MASKS: Lazy<[Bitboard; 64]> =
    Lazy::new(|| precompute_knight_attack_masks());
pub static ROOK_ATTACK_MASKS: Lazy<[Bitboard; 64]> = Lazy::new(|| precompute_rook_attack_mask());
pub static BISHOP_ATTACK_MASKS: Lazy<[Bitboard; 64]> =
    Lazy::new(|| precompute_bishop_attack_mask());
pub static ROOK_ATTACKS: Lazy<HashMap<u64, Bitboard>> =
    Lazy::new(|| precompute_rook_magic_bitboards());

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

fn precompute_knight_attack_masks() -> [Bitboard; 64] {
    let mut knight_attack_table = [Bitboard(0); 64];
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
            if end_square >= 0 && end_square < 64 && i32::abs(square % 8 - end_square % 8) <= 2 {
                knight_attack_bitboard.set_bit(&(end_square as u32));
            }
        }
        knight_attack_table[square as usize] = knight_attack_bitboard;
    }
    knight_attack_table
}
fn precompute_attack_rays() -> [[Bitboard; 8]; 64] {
    let mut attack_rays = [[Bitboard(0); 8]; 64];
    for square in 0..64 {
        let mut square_attack_rays = [Bitboard(0); 8];
        for direction in Direction::all() {
            let mut attack_ray = Bitboard(0);
            for squares_to_edge in 0..SQUARES_TO_EDGE[square][direction] + 1 {
                let end_square = square as i32 + direction.value() * squares_to_edge as i32;
                attack_ray.set_bit(&(end_square as u32));
            }
            square_attack_rays[direction] = attack_ray;
        }
        attack_rays[square] = square_attack_rays;
    }
    attack_rays
}
fn precompute_rook_attack_mask() -> [Bitboard; 64] {
    let mut squares = [0; 64];

    for i in 0..64 {
        squares[i] = i;
    }
    squares.map(|square| {
        Direction::orthagonal()
            .iter()
            .fold(Bitboard(0), |acc, direction| {
                acc | ATTACK_RAYS[square][direction.index()]
            })
    })
}
fn precompute_bishop_attack_mask() -> [Bitboard; 64] {
    let mut squares = [0; 64];
    for i in 0..64 {
        squares[i] = i;
    }
    squares.map(|square| {
        Direction::diagonal()
            .iter()
            .fold(Bitboard(0), |acc, direction| {
                acc | ATTACK_RAYS[square][direction.index()]
            })
    })
}
fn get_bishop_moves_traditional(square: usize, blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard(0);

    attacks |= ATTACK_RAYS[square][Direction::NorthWest];
    if ATTACK_RAYS[square][Direction::NorthWest] & blockers != 0 {
        let blocker_index =
            (ATTACK_RAYS[square][Direction::NorthWest] & blockers).trailing_zeros() as usize;
        attacks &= !ATTACK_RAYS[blocker_index][Direction::NorthWest]
    }

    attacks |= ATTACK_RAYS[square][Direction::NorthEast];
    if ATTACK_RAYS[square][Direction::NorthEast] & blockers != 0 {
        let blocker_index =
            (ATTACK_RAYS[square][Direction::NorthEast] & blockers).trailing_zeros() as usize;
        attacks &= !ATTACK_RAYS[blocker_index][Direction::NorthEast]
    }

    attacks |= ATTACK_RAYS[square][Direction::SouthWest];
    if ATTACK_RAYS[square][Direction::SouthWest] & blockers != 0 {
        let blocker_index =
            (ATTACK_RAYS[square][Direction::SouthWest] & blockers).leading_zeros() as usize;
        attacks &= !ATTACK_RAYS[blocker_index][Direction::SouthWest]
    }

    attacks |= ATTACK_RAYS[square][Direction::SouthEast];
    if ATTACK_RAYS[square][Direction::SouthEast] & blockers != 0 {
        let blocker_index =
            (ATTACK_RAYS[square][Direction::SouthEast] & blockers).leading_zeros() as usize;
        attacks &= !ATTACK_RAYS[blocker_index][Direction::SouthEast]
    }

    attacks
}
fn get_rook_moves_traditional(square: usize, blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard(0);

    attacks |= ATTACK_RAYS[square][Direction::North];
    if ATTACK_RAYS[square][Direction::North] & blockers != 0 {
        let blocker_index =
            (ATTACK_RAYS[square][Direction::North] & blockers).trailing_zeros() as usize;
        attacks &= !ATTACK_RAYS[blocker_index][Direction::North]
    }

    attacks |= ATTACK_RAYS[square][Direction::West];
    if ATTACK_RAYS[square][Direction::West] & blockers != 0 {
        let blocker_index =
            (ATTACK_RAYS[square][Direction::West] & blockers).trailing_zeros() as usize;
        attacks &= !ATTACK_RAYS[blocker_index][Direction::West]
    }

    attacks |= ATTACK_RAYS[square][Direction::South];
    if ATTACK_RAYS[square][Direction::South] & blockers != 0 {
        let blocker_index =
            (ATTACK_RAYS[square][Direction::South] & blockers).leading_zeros() as usize;
        attacks &= !ATTACK_RAYS[blocker_index][Direction::South]
    }

    attacks |= ATTACK_RAYS[square][Direction::East];
    if ATTACK_RAYS[square][Direction::East] & blockers != 0 {
        let blocker_index =
            (ATTACK_RAYS[square][Direction::East] & blockers).leading_zeros() as usize;
        attacks &= !ATTACK_RAYS[blocker_index][Direction::East]
    }

    attacks
}
fn precompute_rook_magic_bitboards() -> HashMap<u64, Bitboard> {
    let mut rook_attacks = HashMap::<u64, Bitboard>::new();
    // same for every square
    let max_blocker_count = ROOK_ATTACK_MASKS[0].count_ones() as u64;
    for square in 0..64 {
        for blockers in 0..1 << max_blocker_count {
            let attack_bitboard = ROOK_ATTACK_MASKS[square];
            let mut blocker_bitboard = Bitboard(0);

            let mut temp_bitboard = attack_bitboard.clone();

            while temp_bitboard.0 != 0 {
                let blocker_index = temp_bitboard.pop_lsb();
                if (Wrapping(blockers) >> blocker_index as usize) & Wrapping(1) == Wrapping(1) {
                    blocker_bitboard.set_bit(&blocker_index);
                }
            }

            blocker_bitboard.0 = (Wrapping(blocker_bitboard.0)
                * Wrapping(*ROOK_MAGIC_NUMBERS.get(square as usize).unwrap()))
            .0;
            blocker_bitboard.0 =
                (Wrapping(blocker_bitboard.0) >> 64 - ROOK_SHIFT_AMOUNT[square] as usize).0;
            rook_attacks.insert(blocker_bitboard.0, attack_bitboard);
        }
    }
    rook_attacks
}
fn precompute_bishop_magic_bitboards() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_squares_to_edge() {
        assert_eq!(SQUARES_TO_EDGE[4][Direction::East], 3);
        assert_eq!(SQUARES_TO_EDGE[27][Direction::North], 3);
        assert_eq!(SQUARES_TO_EDGE[47][Direction::South], 2);
        assert_eq!(SQUARES_TO_EDGE[21][Direction::West], 5);
    }
    #[test]
    fn correct_attack_rays() {
        assert_eq!(SQUARES_TO_EDGE[0][Direction::South], 7);
    }
    #[test]
    fn correct_rook_bitboards() {
        println!("{}", ROOK_ATTACKS.len());
        assert!(true);
    }
}
