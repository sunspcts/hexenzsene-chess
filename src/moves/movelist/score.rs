use crate::{board::Board, heuristics::calc_mvv_lva_heuristic, moves::move_flags};

use super::*;

impl MoveList {
    pub fn score_moves( // Wrapper around score_move to score EVERY move in the list.
        &mut self,
        board: &Board,
        pv_move: Option<Move>,
        tt_move: Option<Move>,
        killers: &[u16],
        history: &[[[i32; 64]; 64]; 2],
    ) {
        let len = self.len as usize;
        for i in 0..len {
            self.scores[i] = score_move(self.moves[i], pv_move, tt_move, killers, history, board);
        }
    }

    pub fn score_qsearch_moves(&mut self, board: &Board) { // Quiescence search doesn't give a shit about PV-Moves, or TT-moves, and history/killers are unneeded.
        let len = self.len as usize;
        for i in 0..len {
            let mv = self.moves[i];
            self.scores[i] = calc_mvv_lva_heuristic(board[mv.from_sq()], mv.captured_piece(board)); 
        }
    }
}

#[inline]
fn score_move(
    mv: Move,
    pv_move: Option<Move>,
    tt_move: Option<Move>,
    killers: &[u16],
    history: &[[[i32; 64]; 64]; 2],
    board: &Board,
) -> i16 {
    if Some(mv) == pv_move {
        return i16::MAX; // PV Moves were the best found at a lower depth. Naturally we want to check them first.
    }
    if Some(mv) == tt_move {
        return i16::MAX - 1; // TT Moves (mostly) caused cutoffs. 
    }
    if mv.flags() & move_flags::QUEEN_PROMO == move_flags::QUEEN_PROMO {
        return 20000; // Queen promotion is always a great call! (Note: underpromotions should be ranked SOMEWHERE, definitely not last.)
    }
    if mv.is_capture() {
        let piece = board[mv.from_sq()];
        let captured = mv.captured_piece(board); // Can't use mv.to_sq() because that is Piece::None after EP.

        let mvv_lva = calc_mvv_lva_heuristic(piece, captured);
        return 10000 + mvv_lva;
    }

    //Rank the killer moves right next to each other.
    if mv.data() == killers[0] {
        return 9000;
    }
    if mv.data() == killers[1] {
        return 8999;
    }

    // Only non-killer quiet moves (and underpromotions right now). Return value from the history table.
    let side = board.game_state.active_side as usize;
    let hist_val = history[side][mv.from_sq() as usize][mv.to_sq() as usize];
    hist_val as i16
}