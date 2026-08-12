//struct, access methods, and impls of operator traits.

#[derive(Copy, Clone, Default, PartialEq)]
pub struct Bitboard(u64); 

/* Bitboard is simply a wrapper around u64, which implements a subset of u64's bitwise operations
but none of its arithmetic operations, mostly for idiot-proofing.

More importantly, it does not directly expose its inner data to the rest of the engine.

Bitboard's field can be accessed in four ways:
- Forward bitscan / Trailing zeros
- Reverse bitscan / Leading zeros
- Popcount
- Its Iterator implementation, which returns the indices of the set bits.

This is mostly, again, for idiot proofing, as otherwise it would be easy to write code such as

let bb_u64 = bitboard.0
// ILLEGAL BITBOARD MANIPULATION
let bitboard = Bitboard(bb_u64)

risking bitboard state corruption.
*/

impl Bitboard {
    pub const fn new(val: u64) -> Self {
        Bitboard(val)
    }

    pub const fn zero() -> Self {
        Bitboard(0)
    }

    pub const fn one() -> Self {
        Bitboard(1)
    }
}

impl std::fmt::Debug for Bitboard {
    // "pretty" prints the bitboard. 

    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "Bitboard:")?;
        for rank in (0..8).rev() {
            for file in 0..8 {
                let shift = (rank * 8) + file;
                let bit = (self.0 >> shift) & 1;
                write!(fmt, "{} ", bit)?;
            }
        writeln!(fmt)?;
        }
        Ok(())
    }
}

// All of these are wrappers around stdlib implementations for u64. 
impl std::ops::Shl<usize> for Bitboard {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        Self(self.0 << rhs)
    }
}

impl std::ops::Shr<usize> for Bitboard {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        Self(self.0 >> rhs)
    }
}

impl std::ops::BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
impl std::ops::BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0
    }
}

impl std::ops::BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, other: Self) {
        self.0 ^= other.0
    }
}

impl std::ops::Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Bitboard {
    pub fn trailing_zeros(&self) -> u32 {
        self.0.trailing_zeros()
    }

    pub fn leading_zeros(&self) -> u32 {
        self.0.leading_zeros()
    }

    pub fn count_ones(&self) -> u32 {
        self.0.count_ones()
    }
}

impl Iterator for Bitboard {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            None
        } else {
            let sq = self.0.trailing_zeros() as u16;
            // really neat trick to pop the LSB, I think it's Kernighan's?
            // if n = 0b10110000, then n - 1 = 0b10101111. Every bit after the least significant bit is one, while the LSB is now 0.
            // n & (n - 1) = 0b10100000.
            self.0 &= self.0 - 1; 
            Some(sq)
        }
    }
}