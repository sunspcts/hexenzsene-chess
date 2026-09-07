use super::*;

use crate::board::Board;
use crate::eval::eval;

impl<'a> Searcher<'a> {
    pub(super) fn quiescence(&mut self, board: &Board, mut context: SearchContext) -> i64 {
        let ply = context.ply().min(MAX_PLY - 1);
        self.pv.clear_ply(ply);

        if self.step_node_and_check() {
            return 0;
        }

        let static_eval = eval(board);
        let mut best_value = static_eval;

        if best_value >= context.beta {
            return best_value;
        }
        if best_value > context.alpha {
            context.alpha = best_value;
        }

        self.generate_and_order_qsearch_moves(board, ply);
        let moves_count = self.move_lists[ply].len();

        for i in 0..moves_count {
            let candidate_move = self.move_lists[ply].pick_best(i);
            if let Some(next_board) = board.make(candidate_move) {
                let score = -self.quiescence(&next_board, context.next_context(0, context.is_pv));

                if self.stopped {
                    return 0;
                }

                if score > best_value {
                    best_value = score;
                }

                if score >= context.beta {
                    return score;
                }
                if score > context.alpha {
                    context.alpha = score;
                }
            }
        }

        best_value
    }
}
