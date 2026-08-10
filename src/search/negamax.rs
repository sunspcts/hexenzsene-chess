use super::*;

use crate::{board::Board};

pub(super) fn negamax(board: &Board, mut context: SearchContext, env: &mut SearchEnv) -> i64 {
    if env.step_node_and_check() { return 0; }

    if context.ply > 0 && env.is_repetition(board.game_state.curr_zobrist_key, board.game_state.half_moves as usize) {
        return 0;
    }

    let in_check = board.is_in_check();
    let depth = if in_check { context.depth + 1 } else { context.depth };

    if depth <= 0 {
        return quiescense(board, context, env);
    }

    let tt_entry = env.tt.get(board.game_state.curr_zobrist_key);
    let tt_move = tt_entry.and_then(|e| e.best_move());

    if context.ply > 0 {
        if let Some(score) = tt_entry.and_then(|e| e.cutoff(context.alpha, context.beta, depth, context.ply)) {
            return score;
        }
    }

    let ply = (context.ply as usize).min(MAX_PLY - 1);
    board.generate_pseudolegal_moves(&mut env.move_lists[ply]);
    env.move_lists[ply].score_moves(board, tt_move, &env.killers[ply], &env.history);
    let moves_count = env.move_lists[ply].len();

    let mut legal_moves_count = 0;
    let mut max_score = i64::MIN;
    let mut best_move = None;
    let old_alpha = context.alpha;

    let mut quiet_moves_tried: [Move; 64] = [Move::new_from_raw(0); 64];
    let mut quiet_count = 0;

    for i in 0..moves_count {
        let candidate_move = env.move_lists[ply].pick_best(i);
        if let Some(next_board) = board.make(candidate_move) {
            legal_moves_count += 1;
            let is_quiet = !candidate_move.is_capture();

            if is_quiet && quiet_count < 64 {
                quiet_moves_tried[quiet_count] = candidate_move;
                quiet_count += 1;
            }

            env.hash_history.push(board.game_state.curr_zobrist_key);

            let score = if legal_moves_count == 1 {
                -negamax(&next_board, context.next_context(depth - 1), env)
            } else {
                let mut s = -negamax(&next_board, context.next_context_null_window(depth - 1), env);
                if s > context.alpha && s < context.beta {
                    s = -negamax(&next_board, context.next_context(depth - 1), env);
                }
                s
            };

            env.hash_history.pop();

            if env.stopped {
                return 0;
            }

            if score > max_score {
                max_score = score;
                best_move = Some(candidate_move);
            }

            if context.update_alpha(score) {
                if is_quiet {
                    if candidate_move.data() != env.killers[ply][0] {
                        env.killers[ply][1] = env.killers[ply][0];
                        env.killers[ply][0] = candidate_move.data();
                    }
                    let side = board.game_state.active_side as usize;
                    history_gravity::update_history_cutoff(
                        &mut env.history,
                        side,
                        candidate_move,
                        &quiet_moves_tried[..quiet_count - 1],
                        depth,
                    );
                }
                break;
            }
        }
    }

    if legal_moves_count == 0 {
        if in_check {
            return -MATE_EVAL + context.ply;
        } else {
            return 0; // Stalemate
        }
    }

    let node_type = context.node_type(max_score, old_alpha);

    if !env.stopped {
        env.tt.store(TTEntry {
            zobrist_key: board.game_state.curr_zobrist_key,
            score: score_to_tt(max_score, context.ply),
            move_data: best_move.map(|m| m.data()).unwrap_or(0),
            depth: depth as i8,
            node_type,
            age: env.age,
        });
    }

    max_score
}