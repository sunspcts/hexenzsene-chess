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
impl MoveList {
    pub fn generate_pseudolegal_moves(&mut self, board: &Board) {
        self.clear();
        let side = board.game_state.active_side;
        self.generate_pawn_moves(board);

        if board.game_state.castling != 0 {
            self.generate_castling_moves(board);
        }

        // Might eventually move these loops into the methods.
        for king in board.piece_bb[side as usize][Piece::King as usize] {
            self.generate_leaper_moves(king, board, Piece::King);
        }

        for knight in board.piece_bb[side as usize][Piece::Knight as usize] {
            self.generate_leaper_moves(knight, board, Piece::Knight);
        }

        for bishop in board.piece_bb[side as usize][Piece::Bishop as usize] {
            self.generate_slider_moves(bishop, board, Piece::Bishop);
        }

        for rook in board.piece_bb[side as usize][Piece::Rook as usize] {
            self.generate_slider_moves(rook, board, Piece::Rook);
        }

        for queen in board.piece_bb[side as usize][Piece::Queen as usize] {
            self.generate_slider_moves(queen, board, Piece::Queen);
        }
    }

    pub fn generate_pseudolegal_caps_promos(&mut self, board: &Board) {
        self.clear();
        let side = board.game_state.active_side;
        self.generate_pawn_caps_promos(board);

        for king in board.piece_bb[side as usize][Piece::King as usize] {
            self.generate_leaper_captures(king, board, Piece::King);
        }

        for knight in board.piece_bb[side as usize][Piece::Knight as usize] {
            self.generate_leaper_captures(knight, board, Piece::Knight);
        }
        for bishop in board.piece_bb[side as usize][Piece::Bishop as usize] {
            self.generate_slider_captures(bishop, board, Piece::Bishop);
        }

        for rook in board.piece_bb[side as usize][Piece::Rook as usize] {
            self.generate_slider_captures(rook, board, Piece::Rook);
        }

        for queen in board.piece_bb[side as usize][Piece::Queen as usize] {
            self.generate_slider_captures(queen, board, Piece::Queen);
        }
    }
}
