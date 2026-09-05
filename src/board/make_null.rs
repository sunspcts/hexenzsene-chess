// Maybe should be in moves/make.rs. But also, it doesnt involve a move! So whatever.

use super::{Board, Side};

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
        if board.game_state.active_side == Side::Black {
            board.game_state.inc_count();
        }
        board.game_state.active_side = enemy;
        board.game_state.en_passant_square = None;

        Some(board)
    }
}