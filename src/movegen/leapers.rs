use super::*;
// See attacks.rs for lookup table generation.
use super::attacks::*;
use crate::{moves::MoveList, piece::Piece};

impl Board {
    pub fn generate_leaper_moves(
        &self,
        from_sq: u16,
        moves: &mut MoveList,
        piece: Piece,
    ) {
        let side = self.game_state.active_side as usize;
        let friendly_pieces = self.side_bb[side];
        //XORing here saves us an (albeit unlikely to be mispredicted) branch.
        let enemy_pieces = self.side_bb[(side) ^ 1];

        // Here we just trust the compiler to inline it properly.
        let raw_attacks = match piece {
            Piece::Knight => KNIGHT_ATTACKS[from_sq as usize],
            Piece::King => KING_ATTACKS[from_sq as usize],
            _ => unreachable!(),
        };

        let valid_moves = raw_attacks & !friendly_pieces;

        // We separate out quiet moves and capture moves because, again, we dont want any branches inside our for loops.
        let captures = valid_moves & enemy_pieces;
        let quiets = valid_moves ^ captures;

        for to_sq in captures {
            moves.push(Move::new(self, from_sq, to_sq, move_flags::CAPTURE, piece));
        }

        for to_sq in quiets {
            moves.push(Move::new(self, from_sq, to_sq, move_flags::QUIET, piece));
        }
    }

    // Used primarily for quiescense search, only generates captures.
    pub fn generate_leaper_captures(
        &self,
        from_sq: u16,
        moves: &mut MoveList,
        piece: Piece,
    ) {
        let side = self.game_state.active_side as usize;
        let enemy_pieces = self.side_bb[side ^ 1];

        let captures = match piece {
            Piece::Knight => KNIGHT_ATTACKS[from_sq as usize],
            Piece::King => KING_ATTACKS[from_sq as usize],
            _ => unreachable!(),
        } & enemy_pieces;

        for to_sq in captures {
            moves.push(Move::new(self, from_sq, to_sq, move_flags::CAPTURE, piece));
        }
    }
}