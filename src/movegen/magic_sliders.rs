use crate::bitboard::Bitboard;
use crate::rng::Xorshift;
use super::legacy_sliders;
use std::sync::OnceLock;

const OUTER_EDGE: Bitboard = Bitboard::new(0xFF818181818181FF);

pub static ROOK_MAGICS: OnceLock<MagicTable> = OnceLock::new();
pub static BISHOP_MAGICS: OnceLock<MagicTable> = OnceLock::new();

#[derive(Copy, Clone, Debug)]
pub struct MagicEntry {
    pub mask: Bitboard,
    pub magic: u64,
    pub shift: u8,
    pub offset: u32,
}

pub struct MagicTable {
    pub entries: [MagicEntry; 64],
    pub attacks: Vec<Bitboard>,
}

impl MagicTable {
    #[inline(always)]
    pub fn get_attacks(&self, blockers: Bitboard, square: u16) -> Bitboard {
        // In any normal code paths, square < 64.
        let entry = unsafe { 
            self.entries.get_unchecked(square as usize) 
        };
        let index = blockers.magic_index(entry.mask, entry.magic, entry.shift as usize);
        unsafe { *self.attacks.get_unchecked(entry.offset as usize + index) }
    }
}

pub fn init_magics() {
    if ROOK_MAGICS.get().is_none() {
        let _ = ROOK_MAGICS.set(init_magic_table(true));
        let _ = BISHOP_MAGICS.set(init_magic_table(false));
    }
}

// ROOK_MAGICS and BISHOP_MAGICS MUST (and will be) initialized on engine start.
// In future, I should be marking these functions as unsafe because I don't trust myself 
// if I'm, say, writing tests to remember to initialize these. But, I'd want to implement
// a proper safe API around them, because I don't want any unsafe blocks in the engine code.
pub fn get_rook_attacks(occupancy: Bitboard, square: u16) -> Bitboard {
    unsafe { ROOK_MAGICS.get().unwrap_unchecked() }.get_attacks(occupancy, square)
}

pub fn get_bishop_attacks(occupancy: Bitboard, square: u16) -> Bitboard {
    unsafe { BISHOP_MAGICS.get().unwrap_unchecked() }.get_attacks(occupancy, square)
}

fn compute_rook_mask(sq: usize) -> Bitboard {
    let rank_mask = Bitboard::new(0xFFu64 << ((sq / 8) * 8));
    let file_mask = Bitboard::new(0x0101010101010101u64 << (sq % 8));
    let edges_to_strip = OUTER_EDGE & !rank_mask & !file_mask;
    legacy_sliders::get_rook_attacks(sq, Bitboard::zero()) & !edges_to_strip
}

fn compute_bishop_mask(sq: usize) -> Bitboard {
    legacy_sliders::get_bishop_attacks(sq, Bitboard::zero()) & !OUTER_EDGE
}

fn generate_occupancies(mask: Bitboard) -> Vec<Bitboard> {
    let mut occupancies = Vec::new();
    let mask_u64 = mask.magic_index(mask, 1, 0) as u64;
    let mut occ = 0u64;
    loop {
        occupancies.push(Bitboard::new(occ));
        occ = occ.wrapping_sub(mask_u64) & mask_u64;
        if occ == 0 {
            break;
        }
    }
    occupancies
}

fn find_magic(sq: usize, mask: Bitboard, is_rook: bool, rng: &mut Xorshift) -> (MagicEntry, Vec<Bitboard>) {
    let occupancies = generate_occupancies(mask);
    let occupancies_count = occupancies.len();
    let mut attacks = Vec::with_capacity(occupancies_count);

    for &occ in &occupancies {
        let att = if is_rook {
            legacy_sliders::get_rook_attacks(sq, occ)
        } else {
            legacy_sliders::get_bishop_attacks(sq, occ)
        };
        attacks.push(att);
    }

    let bits = mask.count_ones() as usize;
    let shift = (64 - bits) as u8;
    let table_size = 1usize << bits;
    let mask_u64 = mask.magic_index(mask, 1, 0) as u64;

    let mut used = vec![Bitboard::zero(); table_size];
    let mut seen = vec![false; table_size];

    loop {
        let magic = rng.magic_candidate();
        let mut fail = false;

        // entropy check
        if (mask_u64.wrapping_mul(magic) & 0xFF00_0000_0000_0000).count_ones() < 6 {
            continue;
        }

        used.fill(Bitboard::zero());
        seen.fill(false);

        for (i, idx) in occupancies.iter().map(|occ| occ.magic_index(mask, magic, shift as usize)).enumerate() {
            if !seen[idx] {
                seen[idx] = true;
                used[idx] = attacks[i];
            } else if used[idx] != attacks[i] {
                fail = true;
                break;
            }
        }

        if !fail {
            return (MagicEntry { mask, magic, shift, offset: 0 }, used);
        }
    }
}

fn init_magic_table(is_rook: bool) -> MagicTable {
    let mut rng = Xorshift::default();
    let mut entries = [MagicEntry {
        mask: Bitboard::zero(),
        magic: 0,
        shift: 0,
        offset: 0,
    }; 64];
    let mut attacks = Vec::new();

    for sq in 0..64 {
        let mask = if is_rook {
            compute_rook_mask(sq)
        } else {
            compute_bishop_mask(sq)
        };

        let (mut entry, square_attacks) = find_magic(sq, mask, is_rook, &mut rng);
        entry.offset = attacks.len() as u32;
        entries[sq] = entry;
        attacks.extend_from_slice(&square_attacks);
    }

    MagicTable {
        entries,
        attacks
    }
}