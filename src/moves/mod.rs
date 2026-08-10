use crate::{board::*, piece::Piece};
#[cfg(test)]
mod tests;

mod make;
mod movelist;
pub mod format;

pub use movelist::MoveList;

// flags from https://www.chessprogramming.org/Encoding_Moves.
#[allow(dead_code)]
pub mod move_flags {
    pub const QUIET: u16               = 0b0000;
    pub const DOUBLE_PAWN_PUSH: u16    = 0b0001;
    pub const KING_CASTLE: u16         = 0b0010;
    pub const QUEEN_CASTLE: u16        = 0b0011;

    pub const CAPTURE: u16             = 0b0100;
    pub const EP_CAPTURE: u16          = 0b0101;

    pub const KNIGHT_PROMO: u16        = 0b1000;
    pub const BISHOP_PROMO: u16        = 0b1001;
    pub const ROOK_PROMO: u16          = 0b1010;
    pub const QUEEN_PROMO: u16         = 0b1011;

    pub const KNIGHT_PROMO_CAPTURE: u16 = 0b1100;
    pub const BISHOP_PROMO_CAPTURE: u16 = 0b1101;
    pub const ROOK_PROMO_CAPTURE: u16   = 0b1110;
    pub const QUEEN_PROMO_CAPTURE: u16  = 0b1111;
}

// CONVENTION
// (indexed from lsb) 
// Bits 0-5: from_sq
// Bits 6-11: to_sq 
// Bits 12-15: move flags (used in make)

#[derive(Clone, Copy, Debug)]
pub struct Move {data: u16}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for Move {}

impl Move {
    // Packs arguments
    pub fn new(_board: &Board, from: u16, to: u16, flags: u16, _piece: Piece) -> Self {
        Move {
            data: {
                (from) | (to << 6) | (flags << 12)
            },
        }
    }

    #[inline] // I'd be concerned if the compiler DIDN'T do this.
    pub fn data(&self) -> u16 {
        self.data
    }

    // Mostly used for initializing non-moves in the movelist, and for transposition tables.
    pub fn new_from_raw(data: u16) -> Self {
        Move {
            data,
        }
    }

    // Helpers for unpacking data field
    pub fn from_sq(self) -> u16 {
        self.data & 0x3F
    }

    pub fn to_sq(self) -> u16 {
        (self.data >> 6) & 0x3F
    }

    pub fn flags(self) -> u16 {
        (self.data >> 12) & 0x3F
    }

    pub fn is_capture(self) -> bool {
        self.flags() & 0b0100 != 0
    }

    pub fn is_promo(self) -> bool {
        self.flags() & 0b1000 != 0
    }

    pub fn captured_piece(self, board: &Board) -> Piece {
        if self.flags() & move_flags::EP_CAPTURE != 0 {
            Piece::Pawn
        } else {
            board[self.to_sq()]
        }
    }
}