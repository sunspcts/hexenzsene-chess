use super::*;

use crate::{moves::MoveList, piece::Piece, bitboard::Bitboard};

// This could surely be less branchy, but I can't think of a nice way to do it.

// CONVENTION
// (indexed from lsb) 
// bit 0 - White Kingside Castling
// bit 1 - White Queenside Castling
// bit 2 - Black Kingside Castling
// bit 3 - Black Queenside Castling

impl Board {
    pub fn generate_castling_moves(
        &self,
        moves: &mut MoveList,
    ) {
        let side = self.game_state.active_side;
        let all_pieces = self.side_bb[0] | self.side_bb[1];

        if side == Side::White {
            //Can Castle Kingside
            if self.game_state.castling & 0b0001 != 0 {
                if (all_pieces & Bitboard::new(0x60)) == Bitboard::zero() {
                    moves.push(Move::new(self, 4, 6, move_flags::KING_CASTLE, Piece::King)); // E1 to G1
                }
            }
            //Can Castle Queenside
            if self.game_state.castling & 0b0010 != 0 {
                if (all_pieces & Bitboard::new(0x0E)) == Bitboard::zero() {
                    moves.push(Move::new(self, 4, 2, move_flags::QUEEN_CASTLE, Piece::King)); // E1 to C1
                }
            }
        } else {
            if self.game_state.castling & 4 != 0 {
                if (all_pieces & Bitboard::new(0x6000000000000000)) == Bitboard::zero() {
                    moves.push(Move::new(self, 60, 62, move_flags::KING_CASTLE, Piece::King)); // E8 to G8
                }
            }
            if self.game_state.castling & 8 != 0 {
                if (all_pieces & Bitboard::new(0x0E00000000000000)) == Bitboard::zero() {
                    moves.push(Move::new(self, 60, 58, move_flags::QUEEN_CASTLE, Piece::King)); // E8 to C8
                }
            }
        }
    }
}