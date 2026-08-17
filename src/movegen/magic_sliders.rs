use std::sync::atomic::{AtomicPtr, Ordering};

use super::legacy_sliders;
use crate::bitboard::Bitboard;
use crate::rng::Xorshift;

const OUTER_EDGE: Bitboard = Bitboard::new(0xFF818181818181FF);

static MAGICS_PTR: AtomicPtr<MagicTable> = AtomicPtr::new(std::ptr::null_mut());

#[derive(Copy, Clone, Debug, Default)]
pub struct MagicEntry {
    pub mask: Bitboard,
    pub magic: u64,
    pub shift: u8,
    pub offset: u32,
}

pub struct MagicTable {
    // 0..64: Bishops, 64..128: Rooks
    pub entries: [MagicEntry; 128],
    pub attacks: Vec<Bitboard>,
}

impl MagicTable {
    /// # SAFETY
    /// entry_idx must be < 128
    #[inline(always)]
    pub unsafe fn get_attacks(&self, blockers: Bitboard, entry_idx: usize) -> Bitboard {
        debug_assert!(
            entry_idx < 128,
            "Magic entry index out of bounds: {}",
            entry_idx
        );
        let entry = unsafe { self.entries.get_unchecked(entry_idx) };
        let index = blockers.magic_index(entry.mask, entry.magic, entry.shift as usize);
        unsafe { *self.attacks.get_unchecked(entry.offset as usize + index) }
    }

    /// # SAFETY
    /// square < 64
    #[inline(always)]
    pub unsafe fn bishop_attacks(&self, blockers: Bitboard, square: u16) -> Bitboard {
        debug_assert!(square < 64, "Bishop square index out of bounds: {}", square);
        unsafe { self.get_attacks(blockers, square as usize) }
    }

    /// # SAFETY
    /// square < 64
    #[inline(always)]
    pub unsafe fn rook_attacks(&self, blockers: Bitboard, square: u16) -> Bitboard {
        debug_assert!(square < 64, "Rook square index out of bounds: {}", square);
        unsafe { self.get_attacks(blockers, 64 + square as usize) }
    }
}

#[inline(always)]
pub fn init_magics() {
    if MAGICS_PTR.load(Ordering::Acquire).is_null() {
        let table = Box::into_raw(Box::new(init_magic_table()));
        if let Err(_existing) = MAGICS_PTR.compare_exchange(
            std::ptr::null_mut(),
            table,
            Ordering::Release,
            Ordering::Acquire,
        ) {
            unsafe {
                drop(Box::from_raw(table));
            }
        }
    }
}

#[inline(always)]
pub fn get_rook_attacks(occupancy: Bitboard, square: u16) -> Bitboard {
    let ptr = MAGICS_PTR.load(Ordering::Acquire);
    unsafe {
        debug_assert!(
            !ptr.is_null(),
            "Magics must be initialized before access"
        );
        (*ptr).rook_attacks(occupancy, square)
    }
}

#[inline(always)]
pub fn get_bishop_attacks(occupancy: Bitboard, square: u16) -> Bitboard {
    let ptr = MAGICS_PTR.load(Ordering::Acquire);
    unsafe {
        debug_assert!(
            !ptr.is_null(),
            "Magics must be initialized before access"
        );
        (*ptr).bishop_attacks(occupancy, square)
    }
}

// Returns ONLY X-Ray attacks on `square` passing through `blockers` to secondary target squares.
#[inline(always)]
pub fn get_rook_xray_attacks(
    occupancy: Bitboard,
    blockers: Bitboard,
    square: u16,
) -> Bitboard {
    let attacks = get_rook_attacks(occupancy, square);
    let filtered_blockers = blockers & attacks;
    let secondary_attacks = get_rook_attacks(occupancy ^ filtered_blockers, square);
    attacks ^ secondary_attacks
}

// Returns ONLY X-Ray attacks on `square` passing through `blockers` to secondary target squares.
#[inline(always)]
pub fn get_bishop_xray_attacks(
    occupancy: Bitboard,
    blockers: Bitboard,
    square: u16,
) -> Bitboard {
    let attacks = get_bishop_attacks(occupancy, square);
    let filtered_blockers = blockers & attacks;
    let secondary_attacks = get_bishop_attacks(occupancy ^ filtered_blockers, square);
    attacks ^ secondary_attacks
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

fn find_magic(
    sq: usize,
    mask: Bitboard,
    is_rook: bool,
    rng: &mut Xorshift,
) -> (MagicEntry, Vec<Bitboard>) {
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

        for (i, idx) in occupancies
            .iter()
            .map(|occ| occ.magic_index(mask, magic, shift as usize))
            .enumerate()
        {
            if !seen[idx] {
                seen[idx] = true;
                used[idx] = attacks[i];
            } else if used[idx] != attacks[i] {
                fail = true;
                break;
            }
        }

        if !fail {
            return (
                MagicEntry {
                    mask,
                    magic,
                    shift,
                    offset: 0,
                },
                used,
            );
        }
    }
}

fn init_magic_table() -> MagicTable {
    let mut rng = Xorshift::default();
    let mut entries = [MagicEntry::default(); 128];
    let mut attacks = Vec::with_capacity(108_000);

    // 1. Bishops (indices 0..64)
    for sq in 0..64 {
        let mask = compute_bishop_mask(sq);
        let (mut entry, square_attacks) = find_magic(sq, mask, false, &mut rng);
        entry.offset = attacks.len() as u32;
        entries[sq] = entry;
        attacks.extend_from_slice(&square_attacks);
    }

    // 2. Rooks (indices 64..128)
    for sq in 0..64 {
        let mask = compute_rook_mask(sq);
        let (mut entry, square_attacks) = find_magic(sq, mask, true, &mut rng);
        entry.offset = attacks.len() as u32;
        entries[64 + sq] = entry;
        attacks.extend_from_slice(&square_attacks);
    }

    attacks.shrink_to_fit();
    MagicTable { entries, attacks }
}
