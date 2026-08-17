use super::*;

use crate::board::Board;

pub(super) fn negamax(board: &Board, mut context: SearchContext, env: &mut SearchEnv) -> i64 {
    let ply = context.ply().min(MAX_PLY - 1);
    env.pv_length[ply] = 0;
    env.pv_table[ply][0] = Move::new_from_raw(0);

    if env.step_node_and_check() {
        return 0;
    } // Have we hit a limit, or has the engine stopped the search?

    // 50 move rule and repetition detection.
    if env.is_draw(board, ply) {
        return 0;
    }

    // Check extensions.
    let in_check = unsafe { board.is_in_check() };
    let depth = context.depth + in_check as i64;
    context.lmr_allowed = depth >= 3 && !in_check;

    // We've reached the depth limit of this iteration.
    if depth <= 0 {
        return quiescense(board, context, env);
    }

    let tt_entry = env.tt.get(board.game_state.curr_zobrist_key); // Probe the TT for this position.
    let tt_move = tt_entry.and_then(|e| e.best_move());

    if ply > 0 {
        // Does the TT move cause a cutoff?
        if let Some(score) =
            tt_entry.and_then(|e| e.cutoff(context.alpha, context.beta, depth, ply as i64))
        {
            return score;
        }
    }

    // Fetch the PV-Move, if we're in the PV.
    let pv_move = if context.is_pv && ply < env.pv_length[0] && env.pv_table[0][ply].data() != 0 {
        Some(env.pv_table[0][ply])
    } else {
        None
    };

    // safety: this is fine. we need to pass a SearchContext to negamax, and the very act of creating a search context generates magic tables.
    unsafe { board.generate_pseudolegal_moves(&mut env.move_lists[ply]) }; // No staged movegen yet. Generate everything.

    env.move_lists[ply].score_moves(board, pv_move, tt_move, &env.killers[ply], &env.history); // Ordering score!
    let moves_count = env.move_lists[ply].len();

    let mut best_move = None;
    let mut max_score = i64::MIN;
    let old_alpha = context.alpha;

    let mut quiet_moves_tried: [Move; 64] = [Move::new_from_raw(0); 64];
    let mut quiet_count = 0;
    let mut move_count = 0;

    for i in 0..moves_count {
        let candidate_move = env.move_lists[ply].pick_best(i);
        if let Some(next_board) = unsafe { board.make(candidate_move) } {
            let is_quiet = !candidate_move.is_capture();
            let is_killer = is_quiet
                && (candidate_move.data() == env.killers[ply][0]
                    || candidate_move.data() == env.killers[ply][1]);

            if is_quiet && quiet_count < 64 {
                // We're gonna give this a malus if another quiet move causes a beta cutoff.
                quiet_moves_tried[quiet_count] = candidate_move;
                quiet_count += 1;
            }

            env.hash_history.push(board.game_state.curr_zobrist_key);
            let score =
                context.search_move(&next_board, depth - 1, move_count, is_quiet, is_killer, env);
            env.hash_history.pop();

            move_count += 1;

            if env.stopped {
                return 0;
            }

            if score > max_score {
                max_score = score;
                best_move = Some(candidate_move);
            }

            if score > context.alpha {
                // New best move, raise alpha!
                context.alpha = score;

                env.update_pv(ply, candidate_move);
            }

            if score >= context.beta {
                // Cutoff! we don't need to look any further down this branch.
                if is_quiet {
                    // We should order this move higher now!
                    if candidate_move.data() != env.killers[ply][0] {
                        env.killers[ply][1] = env.killers[ply][0];
                        env.killers[ply][0] = candidate_move.data();
                    }
                    let side = board.game_state.active_side as usize;
                    history_gravity::update_history_cutoff(
                        &mut env.history,
                        side,
                        candidate_move,                        // Move to incentivise
                        &quiet_moves_tried[..quiet_count - 1], // Moves to penalize
                        depth,
                    );
                }
                break;
            }
        }
    }

    if move_count == 0 {
        // We never found a legal move.
        if in_check {
            return -MATE_EVAL + ply as i64;
        } else {
            return 0; // Stalemate
        }
    }

    if !env.stopped {
        // Best move of the node. We should store it in the TT.
        env.tt.store(TTEntry {
            zobrist_key: board.game_state.curr_zobrist_key,
            score: score_to_tt(max_score, ply as i64),
            move_data: best_move.map(|m| m.data()).unwrap_or(0),
            depth: depth as i8,
            node_type: context.node_type(max_score, old_alpha),
            age: env.age,
        });
    }

    max_score
}
