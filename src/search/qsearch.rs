use super::*;

use crate::board::Board;
use crate::eval::eval;

pub(super) fn quiescense(board: &Board, mut context: SearchContext, env: &mut SearchEnv) -> i64 {
    let ply = (context.ply() as usize).min(MAX_PLY - 1);
    env.pv.clear_ply(ply);

    if env.step_node_and_check() {
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

    env.move_lists[ply].generate_pseudolegal_caps_promos(board);
    env.move_lists[ply].score_qsearch_moves(board);
    let moves_count = env.move_lists[ply].len();

    for i in 0..moves_count {
        let candidate_move = env.move_lists[ply].pick_best(i);
        if let Some(next_board) = board.make(candidate_move) {
            let score = -quiescense(&next_board, context.next_context(0, context.is_pv), env);

            if env.stopped {
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
