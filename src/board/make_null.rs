use super::{Board, Side};
use crate::{bitboard::Bitboard, hashing::ZOBRIST_RANDOMS, piece::Piece};

impl Board {
    pub fn make_null(&self) -> Option<Board> {
        let mut board = *self;
        let side = board.game_state.active_side;
        let enemy = side.flip();

        // if the side to move is in check, we cannot make a null move.
        if board.is_in_check() {
            return None;
        }

        board.game_state.inc_halfmoves();
        if side == Side::Black {
            board.game_state.inc_count();
        }
        board.game_state.active_side = enemy;

        if let Some(old_ep_square) = board.game_state.en_passant_square {
            board.game_state.curr_zobrist_key ^=
                ZOBRIST_RANDOMS[768 + 16 + (old_ep_square % 8) as usize];
            board.game_state.en_passant_square = None;
        }

        board.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[768 + 16 + 8];

        Some(board)
    }

    pub fn has_non_pawn_material(&self, side: Side) -> bool {
        let side_idx = side as usize;
        (self.piece_bb[side_idx][Piece::Knight as usize]
            | self.piece_bb[side_idx][Piece::Bishop as usize]
            | self.piece_bb[side_idx][Piece::Rook as usize]
            | self.piece_bb[side_idx][Piece::Queen as usize])
            != Bitboard::zero()
    }

    pub fn king_pawn_only(&self) -> bool {
        !self.has_non_pawn_material(self.game_state.active_side)
    }
}
