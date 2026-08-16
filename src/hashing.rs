// CONSTANT VALUES

use crate::{
    bitboard::Bitboard,
    board::{Board, Side},
    piece::Piece,
};

pub const ZOBRIST_RANDOMS: [u64; 793] = init_zobrist_random_array();

//evaluated at compile time for quick accesses. we will be using this a lot!
const fn init_zobrist_random_array() -> [u64; 793] {
    let mut arr = [0; 793];
    let mut i = 0;
    let mut state = 0x67676767; // arbitrary.

    while i < 100 {
        state = xorshift(state); // quick mix
        i += 1
    }
    i = 0;
    while i < 793 {
        let new_state = xorshift(state);
        arr[i] = new_state;
        state = new_state;
        i += 1
    }

    arr
}

// Xorshift is totally fine! Doesn't need to be a CSPRNG. Also, Rust allows this to run at compile time, which is very important.
const fn xorshift(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 5;
    state
}
impl Board {
    // Computes the zobrist hash from scratch using bitboards and game state.
    pub fn recompute_zobrist_hash(&mut self) {
        let mut key = 0;
        for sq in 0..64 {
            let piece = self[sq as u16];
            if piece != Piece::None
                && let Some(side) = self.get_side_at(sq as u16)
            {
                key ^= ZOBRIST_RANDOMS[get_piece_zobrist_index(piece, side, sq)];
            }
        }

        key ^= ZOBRIST_RANDOMS[768 + self.game_state.castling as usize];

        if let Some(x) = self.game_state.en_passant_square {
            let file = x % 8;
            key ^= ZOBRIST_RANDOMS[768 + 16 + file as usize];
        }

        if self.game_state.active_side == Side::Black {
            key ^= ZOBRIST_RANDOMS[768 + 16 + 8];
        }

        self.game_state.curr_zobrist_key = key
    }

    // I'm going to replace this with a lookup table soon.
    pub fn update_castling_rights(&mut self, from_sq: u16, to_sq: u16) {
        let old_castling = self.game_state.castling;
        match from_sq {
            4 => self.game_state.castling &= !3,
            60 => self.game_state.castling &= !12,
            7 => self.game_state.castling &= !1,
            0 => self.game_state.castling &= !2,
            63 => self.game_state.castling &= !4,
            56 => self.game_state.castling &= !8,
            _ => (),
        }
        match to_sq {
            7 => self.game_state.castling &= !1,
            0 => self.game_state.castling &= !2,
            63 => self.game_state.castling &= !4,
            56 => self.game_state.castling &= !8,
            _ => (),
        }
        if self.game_state.castling != old_castling {
            self.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[768 + old_castling as usize];
            self.game_state.curr_zobrist_key ^=
                ZOBRIST_RANDOMS[768 + self.game_state.castling as usize];
        }
    }

    // Only called during initial zobrist hash computation, so it's in here.
    fn get_side_at(&self, square: u16) -> Option<Side> {
        let mask = Bitboard::new(1 << square);
        if (self.side_bb[0] & mask) != Bitboard::zero() {
            return Some(Side::White);
        } else if (self.side_bb[1] & mask) != Bitboard::zero() {
            return Some(Side::Black);
        }
        None
    }
}

#[inline]
pub fn get_piece_zobrist_index(piece: Piece, side: Side, sq: usize) -> usize {
    (piece as usize + side as usize * 6) * 64 + sq
}
