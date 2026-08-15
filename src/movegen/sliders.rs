use super::*;

use crate::{bitboard::Bitboard, moves::MoveList, piece::Piece};

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
        let occupancy = self.side_bb[0] | self.side_bb[1];

        // The compiler will hopefully inline this match statement out.
        let raw_attacks = match piece {
            Piece::Rook => magic_sliders::get_rook_attacks(occupancy, from_sq),
            Piece::Bishop => magic_sliders::get_bishop_attacks(occupancy, from_sq),
            // Queen's attack set is identical to the complement of the Rook and Bishop. Nifty!
            Piece::Queen => magic_sliders::get_rook_attacks(occupancy, from_sq) | magic_sliders::get_bishop_attacks(occupancy, from_sq),
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
        let occupancy = self.side_bb[0] | self.side_bb[1];

        let captures = match piece {
            Piece::Rook => magic_sliders::get_rook_attacks(occupancy, from_sq),
            Piece::Bishop => magic_sliders::get_bishop_attacks(occupancy, from_sq),
            // Queen's attack set is identical to the complement of the Rook and Bishop. Nifty!
            Piece::Queen => magic_sliders::get_rook_attacks(occupancy, from_sq) | magic_sliders::get_bishop_attacks(occupancy, from_sq),
            _ => panic!("Piece passed to generate_slider_moves is not a slider!"),
        } & enemy_pieces;

        for to_sq in captures {
            moves.push(Move::new(self, from_sq, to_sq, move_flags::CAPTURE, piece));
        }
    }

    pub fn get_rook_attacks(&self, sq: u16) -> Bitboard {
        let occupancy = self.side_bb[0] | self.side_bb[1];
        magic_sliders::get_rook_attacks(occupancy, sq)
    }

    pub fn get_bishop_attacks(&self, sq: u16) -> Bitboard {
        let occupancy = self.side_bb[0] | self.side_bb[1];
        magic_sliders::get_bishop_attacks(occupancy, sq)
    }
}