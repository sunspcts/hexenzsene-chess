use crate::{board::Board, moves::Move};

use super::lmr::LM_REDUCTIONS_TABLE;
use super::qsearch::quiescense;
use super::tt::{NodeType, TTEntry, score_to_tt};
use super::{MATE_EVAL, MAX_PLY, SearchContext, SearchEnv};

pub(super) fn negamax(board: &Board, mut context: SearchContext, env: &mut SearchEnv) -> i64 {
    let ply = context.ply().min(MAX_PLY - 1);
    env.pv.clear_ply(ply);

    if env.step_node_and_check() || env.is_draw(board, ply) {
        return 0;
    }

    // Check extensions & depth check
    let in_check = board.is_in_check();
    let depth = context.depth + in_check as i64;
    context.lmr_allowed = depth >= 3 && !in_check;

    if depth <= 0 {
        return quiescense(board, context, env);
    }

    // Is this node in the TT?
    let (tt_move, tt_cutoff) =
        probe_tt_cutoff(env, board.game_state.curr_zobrist_key, &context, depth, ply);
    if let Some(score) = tt_cutoff {
        return score;
    }

    // Move Generation & Ordering
    let pv_move = if context.is_pv {
        env.pv.pv_move(ply)
    } else {
        None
    };
    env.move_lists[ply].generate_pseudolegal_moves(board);
    env.move_lists[ply].score_moves(board, pv_move, tt_move, &env.killers.get(ply), &env.history);

    let old_alpha = context.alpha;
    let mut best_move = None;
    let mut max_score = i64::MIN;
    let mut quiet_moves_tried = [Move::new_from_raw(0); 64];
    let mut quiet_count = 0;
    let mut move_count = 0;

    for i in 0..env.move_lists[ply].len() {
        let candidate_move = env.move_lists[ply].pick_best(i);
        let Some(next_board) = board.make(candidate_move) else {
            continue;
        };

        let is_quiet = !candidate_move.is_capture();
        let is_killer = is_quiet && env.killers.is_killer(ply, candidate_move);

        if is_quiet && quiet_count < 64 {
            // We're gonna give this a malus if another quiet move causes a beta cutoff.
            quiet_moves_tried[quiet_count] = candidate_move;
            quiet_count += 1;
        }

        env.hash_history.push(board.game_state.curr_zobrist_key);
        let score = search_move(
            &context,
            &next_board,
            depth - 1,
            move_count,
            is_quiet,
            is_killer,
            env,
        );
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
            context.alpha = score;
            env.pv.update(ply, candidate_move);
        }

        if score >= context.beta {
            if is_quiet {
                update_hist_killers(
                    env,
                    board,
                    ply,
                    candidate_move,
                    &quiet_moves_tried[..quiet_count - 1],
                    depth,
                );
            }
            break;
        }
    }

    if move_count == 0 {
        return if in_check { -MATE_EVAL + ply as i64 } else { 0 };
    }

    store_tt_entry(
        env,
        board.game_state.curr_zobrist_key,
        max_score,
        best_move,
        depth,
        ply,
        context.node_type(max_score, old_alpha),
    );

    max_score
}

#[inline]
fn probe_tt_cutoff(
    env: &SearchEnv,
    zobrist_key: u64,
    context: &SearchContext,
    depth: i64,
    ply: usize,
) -> (Option<Move>, Option<i64>) {
    let tt_entry = env.tt.get(zobrist_key);
    let tt_move = tt_entry.and_then(|e| e.best_move());
    let cutoff_score = if ply > 0 {
        tt_entry.and_then(|e| e.cutoff(context.alpha, context.beta, depth, ply as i64))
    } else {
        None
    };
    (tt_move, cutoff_score)
}

#[inline]
fn update_hist_killers(
    env: &mut SearchEnv,
    board: &Board,
    ply: usize,
    candidate_move: Move,
    quiet_moves_tried: &[Move],
    depth: i64,
) {
    env.killers.add(ply, candidate_move);
    let side = board.game_state.active_side as usize;
    env.history
        .update_cutoff(side, candidate_move, quiet_moves_tried, depth);
}

#[inline]
pub(super) fn store_tt_entry(
    env: &mut SearchEnv,
    zobrist_key: u64,
    score: i64,
    best_move: Option<Move>,
    depth: i64,
    ply: usize,
    node_type: NodeType,
) {
    if !env.stopped {
        env.tt.store(TTEntry {
            zobrist_key,
            score: score_to_tt(score, ply as i64),
            move_data: best_move.map(|m| m.data()).unwrap_or(0),
            depth: depth as i8,
            node_type,
            age: env.age,
        });
    }
}

#[inline(always)]
pub(super) fn search_move(
    context: &SearchContext,
    board: &Board,
    depth: i64,
    move_count: usize,
    is_quiet: bool,
    is_killer: bool,
    env: &mut SearchEnv,
) -> i64 {
    let is_first_move = move_count == 0;
    if is_first_move {
        -negamax(board, context.next_context(depth, context.is_pv), env)
    } else {
        let can_reduce = context.lmr_allowed && is_quiet && !is_killer && move_count >= 3;

        let mut score = if can_reduce {
            let depth_clamp = (context.depth as usize).min(63);
            let move_clamp = move_count.min(63);

            let reduction = LM_REDUCTIONS_TABLE[depth_clamp][move_clamp];
            let lmr_score = -negamax(
                board,
                context.next_context_null_window(depth - reduction),
                env,
            );

            if lmr_score > context.alpha {
                -negamax(board, context.next_context_null_window(depth), env)
            } else {
                lmr_score
            }
        } else {
            -negamax(board, context.next_context_null_window(depth), env)
        };

        if context.is_pv && score > context.alpha && score < context.beta {
            score = -negamax(board, context.next_context(depth, context.is_pv), env);
        }
        score
    }
}
