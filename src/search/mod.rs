mod env;
mod format;
mod history_gravity;
mod killer_heuristic;
mod lmr;
mod negamax;
mod pv;
mod qsearch;
mod tt;

// Imports
use negamax::{search_move, store_tt_entry};

// Private Imports
use format::format_score;
use tt::NodeType;

// Public re-exports
pub use env::{SearchContext, SearchControl, SearchEnv};
pub use history_gravity::HistoryTable;
pub use pv::PvTable;
pub use tt::TT;

use crate::{board::Board, moves::Move};

pub const MATE_EVAL: i64 = 30000;
const NODE_CHECK_INTERVAL_MASK: u64 = 2047; // Check search control every 2048 nodes
pub const MAX_PLY: usize = 256;

fn search_fixed_depth(
    board: &Board,
    depth: i64,
    alpha: i64,
    beta: i64,
    env: &mut SearchEnv,
) -> (i64, Option<Move>) {
    const PLY: usize = 0;
    // Fetch the TT move.
    let tt_move = env
        .tt
        .get(board.game_state.curr_zobrist_key)
        .and_then(|e| e.best_move());

    let pv_move = env.pv.root_move();
    env.pv.clear_ply(0);

    let in_check = board.is_in_check();
    let root_depth = depth + in_check as i64;

    let mut context = SearchContext::new(alpha, beta, root_depth, root_depth >= 3 && !in_check);
    let old_alpha = context.alpha;

    env.move_lists[PLY].generate_pseudolegal_moves(board); // No staged movegen yet. Generate everything.
    env.move_lists[PLY].score_moves(board, pv_move, tt_move, &env.killers.get(PLY), &env.history); // Ordering score!
    let moves_count = env.move_lists[PLY].len();

    let mut best_move = None;
    let mut max_score = i64::MIN;
    let mut move_count = 0;

    for i in 0..moves_count {
        let candidate_move = env.move_lists[PLY].pick_best(i);
        // board.make() returns None if the move is illegal, so this is also our legal move filter.
        let Some(next_board) = board.make(candidate_move) else {
            continue;
        };

        let is_quiet = !candidate_move.is_capture();
        let is_killer = is_quiet && env.killers.is_killer(PLY, candidate_move);

        env.hash_history.push(board.game_state.curr_zobrist_key);
        // Calls safe wrapper around negamax, taking into account the first move.
        let score = search_move(
            &context,
            &next_board,
            root_depth - 1,
            move_count,
            is_quiet,
            is_killer,
            env,
        );
        env.hash_history.pop();

        move_count += 1;

        if env.stopped {
            break;
        }

        if score > max_score {
            max_score = score;
            best_move = Some(candidate_move);
        }

        if score > context.alpha {
            // New best move! Raise alpha.
            context.alpha = score;

            env.pv.update(PLY, candidate_move);
        }

        if score >= context.beta {
            break; // Fail-high cutoff at root
        }
    }

    let best_score = if max_score == i64::MIN {
        context.alpha
    } else {
        max_score
    };

    store_tt_entry(
        env,
        board.game_state.curr_zobrist_key,
        best_score,
        best_move,
        depth,
        0,
        context.node_type(best_score, old_alpha),
    );

    (best_score, best_move)
}

fn search_aspiration(
    board: &Board,
    depth: i64,
    prev_score: i64,
    env: &mut SearchEnv,
) -> (i64, Option<Move>) {
    if depth < 4 {
        return search_fixed_depth(board, depth, -1_000_000, 1_000_000, env);
    }

    let mut delta = 35;
    let mut alpha = (prev_score - delta).max(-1_000_000);
    let mut beta = (prev_score + delta).min(1_000_000);

    loop {
        let (score, best_move) = search_fixed_depth(board, depth, alpha, beta, env);

        if env.stopped {
            return (score, best_move);
        }

        if score <= alpha {
            alpha = (alpha - delta).max(-1_000_000);
            delta += delta / 2;
        } else if score >= beta {
            beta = (beta + delta).min(1_000_000);
            delta += delta / 2;
        } else {
            return (score, best_move);
        }

        if alpha <= -1_000_000 && beta >= 1_000_000 {
            return (score, best_move);
        }
    }
}

pub fn search(board: &Board, max_depth: i64, env: &mut SearchEnv) -> (i64, Option<Move>) {
    let mut global_best_move = None;
    let mut global_best_score = 0;

    for d in 1..=max_depth {
        let (score, best_move) = search_aspiration(board, d, global_best_score, env);

        if env.stopped {
            if global_best_move.is_none() && best_move.is_some() {
                global_best_move = best_move;
                global_best_score = score;
            }
            break;
        }

        if let Some(mv) = best_move {
            global_best_move = Some(mv);
            global_best_score = score;

            if !env.silent {
                let score_str = format_score(score);
                let pv_str = env.pv.format_pv();
                let pv = if pv_str.is_empty() {
                    format!("{mv}")
                } else {
                    pv_str
                };
                println!(
                    "info depth {d} score {score_str} nodes {} pv {pv}",
                    env.nodes_visited
                );
            }
        }
    }

    (global_best_score, global_best_move)
}
