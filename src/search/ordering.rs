use super::Searcher;
use crate::{board::Board, moves::Move};

impl<'a> Searcher<'a> {
    /// Generates pseudolegal moves and scores them for main search (negamax and root search).
    #[inline]
    pub(super) fn generate_and_order_moves(
        &mut self,
        board: &Board,
        tt_move: Option<Move>,
        ply: usize,
        is_pv: bool,
    ) {
        let pv_move = if is_pv { self.pv.pv_move(ply) } else { None };
        self.move_lists[ply].generate_pseudolegal_moves(board);
        self.move_lists[ply].score_moves(
            board,
            pv_move,
            tt_move,
            &self.killers.get(ply),
            &self.history,
        );
    }

    /// Generates pseudolegal captures & promotions and scores them for quiescence search.
    #[inline]
    pub(super) fn generate_and_order_qsearch_moves(&mut self, board: &Board, ply: usize) {
        self.move_lists[ply].generate_pseudolegal_caps_promos(board);
        self.move_lists[ply].score_qsearch_moves(board);
    }
}
