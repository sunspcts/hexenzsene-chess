use super::*;

use crate::{bitboard::Bitboard, moves::MoveList};

// CONVENTION
// (indexed from lsb)
// bit 0 - White Kingside Castling
// bit 1 - White Queenside Castling
// bit 2 - Black Kingside Castling
// bit 3 - Black Queenside Castling

impl MoveList {
    pub fn generate_castling_moves(&mut self, board: &Board) {
        let side = board.game_state.active_side as usize;
        let occupancy = board.side_bb[0] | board.side_bb[1];

        let shift = side * 56;
        let ks_mask = Bitboard::new(0b01100000 << shift); // F1, G1 for white, F8, G8 for black.
        let qs_mask = Bitboard::new(0b00001110 << shift); // B1, C1, D1 for white, B8, C8, D8 for black.
        let king_sq = (4 + shift) as u16;

        let (perm_mask_ks, perm_mask_qs) = (1 << (side * 2), 2 << (side * 2)); // side * 2 = 0 for white, 2 for black.

        if (perm_mask_ks & board.game_state.castling != 0)
            & (occupancy & ks_mask == Bitboard::zero())
        {
            // ya like bitwise comparisons?
            self.push(Move::new_from_raw(
                (king_sq) | ((king_sq + 2) << 6) | (move_flags::KING_CASTLE << 12),
            ))
        }
        if (perm_mask_qs & board.game_state.castling != 0)
            & (occupancy & qs_mask == Bitboard::zero())
        {
            self.push(Move::new_from_raw(
                (king_sq) | ((king_sq - 2) << 6) | (move_flags::QUEEN_CASTLE << 12),
            ))
        }
    }
}
