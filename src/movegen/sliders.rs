use super::*;
use crate::{bitboard::Bitboard, moves::MoveList, piece::Piece};

impl MoveList {
    pub fn generate_slider_moves(&mut self, from_sq: u16, board: &Board, piece: Piece) {
        let side = board.game_state.active_side as usize;
        let friendly_pieces = board.side_bb[side];
        //XORing here saves us an (albeit unlikely to be mispredicted) branch.
        let enemy_pieces = board.side_bb[side ^ 1];
        let occupancy = board.side_bb[0] | board.side_bb[1];

        // The compiler will hopefully inline this match statement out.
        let raw_attacks = match piece {
            Piece::Rook => unsafe { magic_sliders::get_rook_attacks(occupancy, from_sq) },
            Piece::Bishop => unsafe { magic_sliders::get_bishop_attacks(occupancy, from_sq) },
            // Queen's attack set is identical to the complement of the Rook and Bishop. Nifty!
            Piece::Queen => unsafe {
                magic_sliders::get_rook_attacks(occupancy, from_sq)
                    | magic_sliders::get_bishop_attacks(occupancy, from_sq)
            },
            _ => unreachable!("Piece passed to generate_slider_moves is not a slider!"),
        };

        let valid_moves = raw_attacks & !friendly_pieces;
        // We separate out quiet moves and capture moves because, again, we dont want any branches inside our for loops.
        let captures = valid_moves & enemy_pieces;
        let quiets = valid_moves ^ captures;

        for to_sq in captures {
            self.push(Move::new(from_sq, to_sq, move_flags::CAPTURE));
        }

        for to_sq in quiets {
            self.push(Move::new(from_sq, to_sq, move_flags::QUIET));
        }
    }

    // Used primarily for quiescense search, only generates captures.
    pub fn generate_slider_captures(&mut self, from_sq: u16, board: &Board, piece: Piece) {
        let side = board.game_state.active_side as usize;
        let enemy_pieces = board.side_bb[side ^ 1];
        let occupancy = board.side_bb[0] | board.side_bb[1];

        let captures = match piece {
            Piece::Rook => unsafe { magic_sliders::get_rook_attacks(occupancy, from_sq) },
            Piece::Bishop => unsafe { magic_sliders::get_bishop_attacks(occupancy, from_sq) },
            // Queen's attack set is identical to the complement of the Rook and Bishop. Nifty!
            Piece::Queen => unsafe {
                magic_sliders::get_rook_attacks(occupancy, from_sq)
                    | magic_sliders::get_bishop_attacks(occupancy, from_sq)
            },
            _ => panic!("Piece passed to generate_slider_moves is not a slider!"),
        } & enemy_pieces;

        for to_sq in captures {
            self.push(Move::new(from_sq, to_sq, move_flags::CAPTURE));
        }
    }

    pub unsafe fn get_rook_attacks(board: &Board, sq: u16) -> Bitboard {
        let occupancy = board.side_bb[0] | board.side_bb[1];
        unsafe { magic_sliders::get_rook_attacks(occupancy, sq) }
    }

    pub unsafe fn get_bishop_attacks(board: &Board, sq: u16) -> Bitboard {
        let occupancy = board.side_bb[0] | board.side_bb[1];
        unsafe { magic_sliders::get_bishop_attacks(occupancy, sq) }
    }
}
