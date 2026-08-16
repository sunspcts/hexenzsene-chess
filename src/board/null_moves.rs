use super::{Board, Side};
use crate::bitboard::Bitboard;
use crate::hashing::ZOBRIST_RANDOMS;
use crate::piece::Piece;

impl Board {
    pub fn make_null_move(&self) -> Board {
        let mut board = *self;

        if self.game_state.active_side == Side::Black {
            board.game_state.inc_count();
        }

        board.game_state.active_side = self.game_state.active_side.flip();
        board.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[768 + 16 + 8];

        if let Some(sq) = self.game_state.en_passant_square {
            let file = sq % 8;
            board.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[768 + 16 + (file as usize)];
            board.game_state.en_passant_square = None;
        }

        board.game_state.inc_halfmoves();
        board
    }

    pub fn king_pawn_only(&self) -> bool {
        let side = self.game_state.active_side as usize;
        (self.piece_bb[side][Piece::Knight as usize]
            | self.piece_bb[side][Piece::Bishop as usize]
            | self.piece_bb[side][Piece::Rook as usize]
            | self.piece_bb[side][Piece::Queen as usize])
            == Bitboard::zero()
    }
}
