// IMPORTS

mod init;
mod state;
mod null_moves;

use crate::rng::Xorshift;

#[cfg(test)]
mod tests;

use crate::{movegen::attacks::{KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS}, bitboard::Bitboard, piece::Piece};

// CONSTANT VALUES

pub const ZOBRIST_RANDOMS: [u64; 793] = init_zobrist_random_array();

//evaluated at compile time for quick accesses. we will be using this a lot!
const fn init_zobrist_random_array() -> [u64; 793] {
    let mut arr = [0; 793];
    let mut i = 0;
    let mut state = 0x67676767;  // arbitrary.

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

// SIDE ENUM

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Side {
    White = 0,
    Black = 1
}

impl Side {
    pub fn flip(self) -> Self {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
}


// BOARD STRUCT

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Board {
    pub piece_bb: [[Bitboard; 6]; 2],
    pub side_bb: [Bitboard; 2],
    pub game_state: state::GameState,
    mailbox: [Piece; 64]
}

// VERY useful to be able to just index the board like this.
impl std::ops::Index<u16> for Board {
    type Output = Piece;

    fn index(&self, sq: u16) -> &Self::Output {
        &self.mailbox[sq as usize]
    }
}

impl std::ops::IndexMut<u16> for Board {
    fn index_mut(&mut self, sq: u16) -> &mut Self::Output {
        &mut self.mailbox[sq as usize]
    }
}

impl Board {

    // Only called during initial zobrist hash computation.
    fn get_side_at(&self, square: u16) -> Option<Side> {
        let mask = Bitboard::new(1 << square);
        if (self.side_bb[0] & mask) != Bitboard::zero() {
            return Some(Side::White)
        } else if (self.side_bb[1] & mask) != Bitboard::zero() {
            return Some(Side::Black)
        }
        None
    }

    // TODO: Rewrite this maybe? Might be unnecessary. Profiling will tell me :)
    pub fn is_attacked(&self, square: u16, attacker_side: Side) -> bool {
        let attacker = attacker_side as usize;
        let defender = (attacker_side as usize) ^ 1;

        // This function assumes a "super-piece" on square, generates attacks from the super-piece rather than from the opponent pieces themselves.
        // Saves extremely costly iteration over every piece on the board .

        let enemy_knights = self.piece_bb[attacker][Piece::Knight as usize];
        if (KNIGHT_ATTACKS[square as usize] & enemy_knights) != Bitboard::zero() {
            return true;
        }

        let enemy_king = self.piece_bb[attacker][Piece::King as usize];
        if (KING_ATTACKS[square as usize] & enemy_king) != Bitboard::zero() {
            return true;
        }

        let enemy_pawns = self.piece_bb[attacker][Piece::Pawn as usize];
        if (PAWN_ATTACKS[defender][square as usize] & enemy_pawns) != Bitboard::zero() {
            return true;
        }

        let diagonal_attackers = self.piece_bb[attacker][Piece::Bishop as usize] | self.piece_bb[attacker][Piece::Queen as usize];
        if (self.get_bishop_attacks(square) & diagonal_attackers) != Bitboard::zero() {
            return true;
        }

        let orthogonal_attackers = self.piece_bb[attacker][Piece::Rook as usize] | self.piece_bb[attacker][Piece::Queen as usize];
        if (self.get_rook_attacks(square) & orthogonal_attackers) != Bitboard::zero() {
            return true;
        }
        false
    }

    // Computes the zobrist hash from scratch using bitboards and game state.
    fn recompute_zobrist_hash(&mut self) {
        let mut key = 0;
        for sq in 0..64 {
            let piece = self[sq as u16];
            if piece != Piece::None && let Some(side) = self.get_side_at(sq as u16) {
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
            _ => ()
        }
        match to_sq {
            7 => self.game_state.castling &= !1,
            0 => self.game_state.castling &= !2,
            63 => self.game_state.castling &= !4,
            56 => self.game_state.castling &= !8,
            _ => ()
        }
        if self.game_state.castling != old_castling {
            self.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[768 + old_castling as usize];
            self.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[768 + self.game_state.castling as usize];
        }
    }

    // I can probably use a simplified function compared to is_attacked for this.
    pub fn is_in_check(&self) -> bool {
        let side = self.game_state.active_side;
        let king_bb = self.piece_bb[side as usize][Piece::King as usize];
        let king_square = king_bb.trailing_zeros() as u16;

        self.is_attacked(king_square, side.flip())
    }

    pub fn side_to_move_multiplier(&self) -> i64 {
        match self.game_state.active_side {
            Side::White => 1,
            Side::Black => -1
        }
    }
}

// ZOBRIST HASHING

#[inline]
pub fn get_piece_zobrist_index(piece: Piece, side: Side, sq: usize) -> usize {
    (piece as usize + side as usize * 6) * 64 + sq
}

