use crate::bitboard::Bitboard;
use super::attacks::{RAYS, IS_POS_DIR};

pub fn get_ray_attacks(sq: usize, dir: usize, occupancy: Bitboard) -> Bitboard {
    let ray = RAYS[dir][sq];
    let blockers = ray & occupancy;

    if blockers == Bitboard::zero() {
        return ray;
    }

    let blocker_sq_pos = blockers.trailing_zeros() as usize;
    let blocker_sq_neg = 63 ^ (blockers.leading_zeros() as usize);
    let mask = IS_POS_DIR[dir];
    let blocker_sq = (blocker_sq_pos & mask) | (blocker_sq_neg & !mask);

    let shadow = RAYS[dir][blocker_sq];
    ray ^ shadow
}

pub fn get_rook_attacks(sq: usize, occupancy: Bitboard) -> Bitboard {
    let mut raw_attacks = Bitboard::zero();
    for dir in 0..4 {
        raw_attacks |= get_ray_attacks(sq, dir, occupancy);
    }
    raw_attacks
}

pub fn get_bishop_attacks(sq: usize, occupancy: Bitboard) -> Bitboard {
    let mut raw_attacks = Bitboard::zero();
    for dir in 4..8 {
        raw_attacks |= get_ray_attacks(sq, dir, occupancy);
    }
    raw_attacks
}
