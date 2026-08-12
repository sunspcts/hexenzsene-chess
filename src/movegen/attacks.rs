use crate::{bitboard::Bitboard};

//Mostly constant lookup table initializations.

pub const KING_ATTACKS: [Bitboard; 64] = init_leaper_attacks(&DIR_OFFSETS);
pub const KNIGHT_ATTACKS: [Bitboard; 64] = init_leaper_attacks(&KNIGHT_OFFSETS);
pub const PAWN_ATTACKS: [[Bitboard; 64]; 2] = init_pawn_attacks();
pub const RAYS: [[Bitboard; 64]; 8] = init_ray_lookup();

const DIR_OFFSETS: [(i8, i8); 8] = [
    (1, 0), // N
    (-1, 0), // S
    (0, 1), // E
    (0, -1), // W
    (1, 1), // NE
    (1, -1), // NW
    (-1, 1), // SE
    (-1, -1), // SW
];

const KNIGHT_OFFSETS: [(i8, i8); 8] = [
    (2, 1), // N + NE
    (1, 2), // E + NE
    (-1, 2), // E + SE
    (-2, 1), // S + SE
    (-2, -1), // S + SW
    (-1, -2), // W + SW
    (1, -2), // W + NW
    (2, -1), // N + NW
];

const A_FILE: u64 = 0x0101010101010101;
const H_FILE: u64 = 0x8080808080808080;

const IS_POS_DIR: [usize; 8] = [usize::MAX, 0, usize::MAX, 0, usize::MAX, usize::MAX, 0, 0];

const fn init_leaper_attacks(offsets: &[(i8, i8)]) -> [Bitboard; 64] {
    let mut attacks = [Bitboard::zero(); 64];
    let mut sq: usize = 0;

    while sq < 64 {
        let mut bb = 0; // We generate one bitboard and mutate it in place.
        
        let rank = (sq / 8) as i8;  // Cheaper ways to do these both, but it's compile time evaluated. Who cares?
        let file = (sq % 8) as i8;

        let mut i = 0;
        while i < offsets.len() {
            let (dr, df) = offsets[i];
            let (r, f) = (rank + dr, file + df); // Rank and file with offsets applied.

            // Check we're not trying to place a piece off the board. 
            // If we tried to place a piece on, for example "I6", it would wrap around and place on A7.
            if r >= 0 && f >= 0 && r < 8 && f < 8 {
                bb |= 1u64 << (r * 8 + f); // 1 << x is the bitboard for a piece on square x. (zero indexed square numbering)
            }

            i += 1;
        }

        attacks[sq] = Bitboard::new(bb); 
        sq += 1;
    }

    attacks
}

const fn init_pawn_attacks() -> [[Bitboard; 64]; 2] {
    // Pawns need two separate lookup tables for white and black. 
    // These tables aren't actually used in the pawn movegen proper, but they're useful to have.
    let mut attacks = [[Bitboard::zero(); 64]; 2];
    let mut sq = 0;

    while sq < 64 {
        let bb = 1u64 << sq; // Placing a temporary pawn on the square we're generating the table for.

        // Both more simple and more complex than the leaper shifts. No need to do bounds checking,
        // we know that pawns on the A file cannot attack left, and pawns on the H file cannot attack right.
        // We just mask our temporary bitboard before doing the attack generations.

        let white_attacks = Bitboard::new(((bb & !A_FILE) << 7) | ((bb & !H_FILE) << 9));
        attacks[0][sq] = white_attacks;

        let black_attacks = Bitboard::new(((bb & !H_FILE) >> 7) | ((bb & !A_FILE) >> 9));
        attacks[1][sq] = black_attacks;

        sq += 1;
    }

    attacks
}

const fn init_ray_lookup() -> [[Bitboard; 64]; 8] {
    // One bitboard for each square, with the ray to the edge of the board in that direction.
    let mut rays = [[Bitboard::zero(); 64]; 8];
    let mut sq = 0;

    while sq < 64 {
        let rank = (sq / 8) as i8;
        let file = (sq % 8) as i8;
        let mut dir = 0;

        while dir < 8 {
            let mut ray_bb = 0;
            let (dr, df) = DIR_OFFSETS[dir];
            let (mut r, mut f) = (rank + dr, file + df);

            while r >= 0 && f >= 0 && r < 8 && f < 8 {
                // Step in the ray direction, then add that square to the bitboard. Stop when you reach the edge of the board.
                ray_bb |= 1u64 << (r * 8 + f);
                r += dr;
                f += df;
            }

            rays[dir][sq] = Bitboard::new(ray_bb);
            dir += 1;
        }
        sq += 1;
    }
    rays
}

// This isn't a constant! I thought this was the constant file! 
pub fn get_ray_attacks(sq: u16, dir: usize, occupancy: Bitboard) -> Bitboard {

    // Blockers is a bitboard of ANY piece that could block our slider.
    let ray = RAYS[dir][sq as usize];
    let blockers = ray & occupancy;

    if blockers == Bitboard::zero() {
        return ray
    }
    
    /* In our quest to avoid any branching whatsoever, we reach this utter bullshit. 
    N, E, NE, and NW are considered "positive" ray directions, since to move a piece 
    in these directions we shift our bitboard left.
    Naturally, S, W, SE, and SW are considered negative, due to the right bitshift.
    
    In the case of a positive ray, the first blocking piece is the first piece on a 
    square with index HIGHER than our own, so we can find its index by doing a forward
    bitscan. Similarly, we find the index of the first blocking piece for a negative ray
    by doing a backward bitscan.*/ 

    let blocker_sq_pos = blockers.trailing_zeros() as usize;
    let blocker_sq_neg = 63 ^ (blockers.leading_zeros() as usize); 

    // This lookup table assigns positive rays an identity mask, and negative rays a null mask.
    let mask = IS_POS_DIR[dir];

    /*  Selects blocker square. If the ray is positive, mask is identity, and !mask is null, so
    blocker_sq = blocker_sq_pos. If negative the opposite is true. */
    let blocker_sq = (blocker_sq_pos & mask) | (blocker_sq_neg & !mask);
    
    // Cast a ray from the blocking piece.
    let shadow = RAYS[dir][blocker_sq]; 
    ray ^ shadow
}