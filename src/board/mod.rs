// IMPORTS

mod init;
mod null_moves;
mod state;

#[cfg(test)]
mod tests;

use crate::{
    bitboard::Bitboard,
    movegen::{
        attacks::{KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS},
        magic_sliders,
    },
    piece::Piece,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Side {
    White = 0,
    Black = 1,
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
    mailbox: [Piece; 64],
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
    #[inline(always)]
    /// # SAFETY
    /// MAGICS_PTR must be initialized.
    pub unsafe fn is_attacked(&self, square: u16, attacker_side: Side) -> bool {
        let attacker = attacker_side as usize;
        let defender = (attacker_side as usize) ^ 1;
        let sq = square as usize;
        let attacker_pieces = self.piece_bb[attacker];

        // This function assumes a "super-piece" on square, generates attacks from the super-piece rather than from the opponent pieces themselves.
        // Saves extremely costly iteration over every piece on the board .

        let enemy_pawns = attacker_pieces[Piece::Pawn as usize];
        if (unsafe { *PAWN_ATTACKS.get_unchecked(defender).get_unchecked(sq) } & enemy_pawns)
            != Bitboard::zero()
        {
            return true;
        }

        let enemy_knights = attacker_pieces[Piece::Knight as usize];
        if (unsafe { *KNIGHT_ATTACKS.get_unchecked(sq) } & enemy_knights) != Bitboard::zero() {
            return true;
        }

        let occupancy = self.side_bb[0] | self.side_bb[1];
        let enemy_queens = attacker_pieces[Piece::Queen as usize];
        let diagonal_attackers = attacker_pieces[Piece::Bishop as usize] | enemy_queens;
        if diagonal_attackers != Bitboard::zero() {
            if (unsafe { magic_sliders::get_bishop_attacks(occupancy, square) }
                & diagonal_attackers)
                != Bitboard::zero()
            {
                return true;
            }
        }

        let orthogonal_attackers = attacker_pieces[Piece::Rook as usize] | enemy_queens;
        if orthogonal_attackers != Bitboard::zero() {
            if (unsafe { magic_sliders::get_rook_attacks(occupancy, square) }
                & orthogonal_attackers)
                != Bitboard::zero()
            {
                return true;
            }
        }

        let enemy_king = attacker_pieces[Piece::King as usize];
        if (unsafe { *KING_ATTACKS.get_unchecked(sq) } & enemy_king) != Bitboard::zero() {
            return true;
        }

        false
    }
    // I can probably use a simplified function compared to is_attacked for this.
    #[inline(always)]
    pub unsafe fn is_in_check(&self) -> bool {
        let side = self.game_state.active_side;
        let king_bb = self.piece_bb[side as usize][Piece::King as usize];
        let king_square = king_bb.trailing_zeros() as u16;
        unsafe { self.is_attacked(king_square, side.flip()) }
    }

    pub fn side_to_move_multiplier(&self) -> i64 {
        match self.game_state.active_side {
            Side::White => 1,
            Side::Black => -1,
        }
    }

    /// Returns a bitboard containing all friendly pieces that are pinned to the king.
    pub fn pinned_bitboard(&self, side: Side) -> Bitboard {
        let enemy = side.flip();
        let king_sq = (self.piece_bb[side as usize][Piece::King as usize]).trailing_zeros() as u16;
        if king_sq >= 64 {
            return Bitboard::zero();
        }

        let occupancy = self.side_bb[0] | self.side_bb[1];
        let friendly_pieces = self.side_bb[side as usize];
        let mut pinned_pieces = Bitboard::zero();

        let enemy_rooks_and_queens = self.piece_bb[enemy as usize][Piece::Rook as usize]
            | self.piece_bb[enemy as usize][Piece::Queen as usize];

        if enemy_rooks_and_queens != Bitboard::zero() {
            let mut pinners = unsafe {
                magic_sliders::get_rook_xray_attacks(occupancy, friendly_pieces, king_sq)
            } & enemy_rooks_and_queens;
            while pinners != Bitboard::zero() {
                let pinner_sq = pinners.trailing_zeros() as u16;
                let line_between = unsafe {
                    magic_sliders::get_rook_attacks(Bitboard::zero(), king_sq)
                        & magic_sliders::get_rook_attacks(Bitboard::zero(), pinner_sq)
                };
                pinned_pieces |= line_between & friendly_pieces;
                pinners ^= Bitboard::new(1u64 << pinner_sq);
            }
        }

        let enemy_bishops_and_queens = self.piece_bb[enemy as usize][Piece::Bishop as usize]
            | self.piece_bb[enemy as usize][Piece::Queen as usize];

        if enemy_bishops_and_queens != Bitboard::zero() {
            let mut pinners = unsafe {
                magic_sliders::get_bishop_xray_attacks(occupancy, friendly_pieces, king_sq)
            } & enemy_bishops_and_queens;
            while pinners != Bitboard::zero() {
                let pinner_sq = pinners.trailing_zeros() as u16;
                let line_between = unsafe {
                    magic_sliders::get_bishop_attacks(Bitboard::zero(), king_sq)
                        & magic_sliders::get_bishop_attacks(Bitboard::zero(), pinner_sq)
                };
                pinned_pieces |= line_between & friendly_pieces;
                pinners ^= Bitboard::new(1u64 << pinner_sq);
            }
        }

        pinned_pieces
    }
}
