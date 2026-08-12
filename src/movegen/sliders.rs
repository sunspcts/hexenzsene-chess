use super::*;
// See attacks.rs for lookup table generation.
use super::attacks::*;

use crate::{bitboard::Bitboard, moves::MoveList, piece::Piece};

// We currently use raycasting to generate sliding piece moves.
// A future version of the engine will probably use magic bitboards, but I didn't fully understand them at the time of implementing movegen.

impl Board {
    pub fn generate_slider_moves(
        &self,
        from_sq: u16,
        moves: &mut MoveList,
        piece: Piece,
    ) {
        let side = self.game_state.active_side as usize;
        let friendly_pieces = self.side_bb[side];
        //XORing here saves us an (albeit unlikely to be mispredicted) branch.
        let enemy_pieces = self.side_bb[side ^ 1];

        // The compiler will hopefully inline this match statement out.
        let raw_attacks = match piece {
            Piece::Rook => self.get_rook_attacks(from_sq),
            Piece::Bishop => self.get_bishop_attacks(from_sq),
            // Queen's attack set is identical to the complement of the Rook and Bishop. Nifty!
            Piece::Queen => self.get_rook_attacks(from_sq) | self.get_bishop_attacks(from_sq),
            _ => unreachable!("Piece passed to generate_slider_moves is not a slider!"),
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
    pub fn generate_slider_captures(
        &self,
        from_sq: u16,
        moves: &mut MoveList,
        piece: Piece,
    ) {
        let side = self.game_state.active_side as usize;
        let enemy_pieces = self.side_bb[side ^ 1];

        let captures = match piece {
            Piece::Rook => self.get_rook_attacks(from_sq),
            Piece::Bishop => self.get_bishop_attacks(from_sq),
            // Queen's attack set is identical to the complement of the Rook and Bishop. Nifty!
            Piece::Queen => self.get_rook_attacks(from_sq) | self.get_bishop_attacks(from_sq),
            _ => panic!("Piece passed to generate_slider_moves is not a slider!"),
        } & enemy_pieces;

        for to_sq in captures {
            moves.push(Move::new(self, from_sq, to_sq, move_flags::CAPTURE, piece));
        }
    }

    // Essentially just wrappers around get_ray_attacks. 
    pub fn get_rook_attacks(&self, sq: u16) -> Bitboard {
        let dirs = [0,1,2,3]; // N, S, E, W
        let occupancy = self.side_bb[0] | self.side_bb[1];

        let mut raw_attacks: Bitboard = Bitboard::zero();

        for dir in dirs {
            raw_attacks |= get_ray_attacks(sq, dir, occupancy)
        }

        raw_attacks
    }

    pub fn get_bishop_attacks(&self, sq: u16) -> Bitboard {
        let dirs = [4,5,6,7]; // NE, NW, SE, SW
        let occupancy = self.side_bb[0] | self.side_bb[1];

        let mut raw_attacks: Bitboard = Bitboard::zero();

        for dir in dirs {
            raw_attacks |= get_ray_attacks(sq, dir, occupancy)
        }

        raw_attacks
    }
}