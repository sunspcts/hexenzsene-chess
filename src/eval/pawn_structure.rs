use crate::bitboard::Bitboard;

const FILE_A: u64 = 0x0101010101010101;
const FILE_B: u64 = 0x0202020202020202;
const FILE_C: u64 = 0x0404040404040404;
const FILE_D: u64 = 0x0808080808080808;
const FILE_E: u64 = 0x1010101010101010;
const FILE_F: u64 = 0x2020202020202020;
const FILE_G: u64 = 0x4040404040404040;
const FILE_H: u64 = 0x8080808080808080;

const ADJACENT_FILE_MASKS: [Bitboard; 8] = [
    Bitboard::new(FILE_B),
    Bitboard::new(FILE_A | FILE_C),
    Bitboard::new(FILE_B | FILE_D),
    Bitboard::new(FILE_C | FILE_E),
    Bitboard::new(FILE_D | FILE_F),
    Bitboard::new(FILE_E | FILE_G),
    Bitboard::new(FILE_F | FILE_H),
    Bitboard::new(FILE_G)
];

pub const FILE_MASKS: [Bitboard; 8] = [
    Bitboard::new(FILE_A),
    Bitboard::new(FILE_B),
    Bitboard::new(FILE_C),
    Bitboard::new(FILE_D),
    Bitboard::new(FILE_E),
    Bitboard::new(FILE_F),
    Bitboard::new(FILE_G),
    Bitboard::new(FILE_H),
];

const fn generate_passed_pawn_mask(color: usize, sq: usize) -> u64 {
    let rank = sq / 8;
    let file = sq % 8;
    let mut mask: u64 = 0;

    if color == 0 {
        let mut r = rank + 1;
        while r < 8 {
            mask |= 1u64 << (r * 8 + file);
            if file > 0 {
                mask |= 1u64 << (r * 8 + file - 1);
            }
            if file < 7 {
                mask |= 1u64 << (r * 8 + file + 1);
            }
            r += 1;
        }
    } else {
        let mut r = 0;
        while r < rank {
            mask |= 1u64 << (r * 8 + file);
            if file > 0 {
                mask |= 1u64 << (r * 8 + file - 1);
            }
            if file < 7 {
                mask |= 1u64 << (r * 8 + file + 1);
            }
            r += 1;
        }
    }
    mask
}

pub const PASSED_PAWN_MASKS: [[Bitboard; 64]; 2] = {
    let mut masks = [[Bitboard::zero(); 64]; 2];
    let mut color = 0;
    while color < 2 {
        let mut sq = 0;
        while sq < 64 {
            masks[color][sq] = Bitboard::new(generate_passed_pawn_mask(color, sq));
            sq += 1;
        }
        color += 1;
    }
    masks
};

#[inline]
pub fn is_isolated(sq: u16, friendly_pawns: Bitboard) -> bool {
    let file = (sq & 0b0111) as usize;
    (friendly_pawns & ADJACENT_FILE_MASKS[file]) == Bitboard::zero()
}

#[inline]
pub fn is_passed(sq: u16, color: usize, enemy_pawns: Bitboard) -> bool {
    (PASSED_PAWN_MASKS[color][sq as usize] & enemy_pawns) == Bitboard::zero()
}