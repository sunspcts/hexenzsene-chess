pub mod attacks;
mod castling;
mod leapers;
pub mod legacy_sliders;
pub mod magic_sliders;
mod pawns;
mod sliders;
use crate::{moves::*, piece::Piece};

use crate::board::{Board, Side};

//Pseudolegal move generation. Legality checking is done at the make_move() stage.
impl Board {
    pub unsafe fn generate_pseudolegal_moves(&self, moves: &mut MoveList) {
        moves.clear();
        let side = self.game_state.active_side;
        self.generate_pawn_moves(moves);

        if self.game_state.castling != 0 {
            self.generate_castling_moves(moves);
        }

        // Might eventually move these loops into the methods.
        for king in self.piece_bb[side as usize][Piece::King as usize] {
            self.generate_leaper_moves(king, moves, Piece::King);
        }

        for knight in self.piece_bb[side as usize][Piece::Knight as usize] {
            self.generate_leaper_moves(knight, moves, Piece::Knight);
        }
        unsafe {
            for bishop in self.piece_bb[side as usize][Piece::Bishop as usize] {
                self.generate_slider_moves(bishop, moves, Piece::Bishop);
            }

            for rook in self.piece_bb[side as usize][Piece::Rook as usize] {
                self.generate_slider_moves(rook, moves, Piece::Rook);
            }

            for queen in self.piece_bb[side as usize][Piece::Queen as usize] {
                self.generate_slider_moves(queen, moves, Piece::Queen);
            }
        }
    }

    pub unsafe fn generate_pseudolegal_caps_promos(&self, moves: &mut MoveList) {
        moves.clear();
        let side = self.game_state.active_side;
        self.generate_pawn_caps_promos(moves);

        for king in self.piece_bb[side as usize][Piece::King as usize] {
            self.generate_leaper_captures(king, moves, Piece::King);
        }

        for knight in self.piece_bb[side as usize][Piece::Knight as usize] {
            self.generate_leaper_captures(knight, moves, Piece::Knight);
        }
        unsafe {
            for bishop in self.piece_bb[side as usize][Piece::Bishop as usize] {
                self.generate_slider_captures(bishop, moves, Piece::Bishop);
            }

            for rook in self.piece_bb[side as usize][Piece::Rook as usize] {
                self.generate_slider_captures(rook, moves, Piece::Rook);
            }

            for queen in self.piece_bb[side as usize][Piece::Queen as usize] {
                self.generate_slider_captures(queen, moves, Piece::Queen);
            }
        }
    }
}
