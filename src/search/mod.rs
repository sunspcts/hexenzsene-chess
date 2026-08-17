// This file is a fucking mess.

mod env;
mod format;
mod history_gravity;
mod lmr;
mod negamax;
mod qsearch;
mod tt;

// Imports
use negamax::negamax;
use qsearch::quiescense;

// Private Imports
use format::format_score;
use lmr::LM_REDUCTIONS_TABLE;
use tt::{NodeType, TTEntry, score_to_tt};

// Public re-exports
pub use env::{SearchContext, SearchControl, SearchEnv};
pub use tt::TT;

use crate::{board::Board, movegen::magic_sliders::init_magics, moves::Move};

pub const MATE_EVAL: i64 = 30000;
const NODE_CHECK_INTERVAL_MASK: u64 = 2047; // Check search control every 2048 nodes
pub const MAX_PLY: usize = 256;

/// # SAFETY
/// Calls unsafe fn `board.is_in_check()`` before initializing context, which requires `MAGICS_PTR` to be initialized.
unsafe fn search_fixed_depth(
    board: &Board,
    depth: i64,
    env: &mut SearchEnv,
) -> (i64, Option<Move>) {
    // Fetch the TT move.
    let tt_move = env
        .tt
        .get(board.game_state.curr_zobrist_key)
        .and_then(|e| e.best_move());
    let ply = 0;

    let pv_move = if env.pv_length[0] > 0 && env.pv_table[0][0].data() != 0 {
        Some(env.pv_table[0][0])
    } else {
        None
    };

    env.pv_length[0] = 0;
    env.pv_table[0][0] = Move::new_from_raw(0);

    let in_check = unsafe { board.is_in_check() };
    let root_depth = depth + in_check as i64;

    let mut context = SearchContext::new_full_window(root_depth, root_depth >= 3 && !in_check);

    unsafe { board.generate_pseudolegal_moves(&mut env.move_lists[ply]) }; // No staged movegen yet. Generate everything.
    env.move_lists[ply].score_moves(board, pv_move, tt_move, &env.killers[ply], &env.history); // Ordering score!
    let moves_count = env.move_lists[ply].len();

    let mut best_move = None;
    // PV Search treats the first move differently, as it's the only move that's searched with a full window by default.
    let mut move_count = 0;

    for i in 0..moves_count {
        let candidate_move = env.move_lists[ply].pick_best(i);
        // board.make() returns None if the move is illegal, so this is also our legal move filter.
        if let Some(next_board) = unsafe { board.make(candidate_move) } {
            let is_quiet = !candidate_move.is_capture();
            let is_killer = is_quiet
                && (candidate_move.data() == env.killers[ply][0]
                    || candidate_move.data() == env.killers[ply][1]);

            env.hash_history.push(board.game_state.curr_zobrist_key);
            // Calls safe wrapper around negamax, taking into account the first move.
            let score = context.search_move(
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

            if score > context.alpha {
                // New best move! Raise alpha.
                best_move = Some(candidate_move);
                context.alpha = score;

                env.update_pv(ply, candidate_move);
            }
        }
    }

    if !env.stopped && best_move.is_some() {
        env.tt.store(TTEntry {
            zobrist_key: board.game_state.curr_zobrist_key,
            score: score_to_tt(context.alpha, 0),
            move_data: best_move.map(|m| m.data()).unwrap_or(0),
            depth: depth as i8,
            node_type: NodeType::Exact,
            age: env.age,
        });
    }

    (context.alpha, best_move)
}

pub fn search(board: &Board, max_depth: i64, env: &mut SearchEnv) -> (i64, Option<Move>) {
    init_magics();
    let mut global_best_move = None;
    let mut global_best_score = 0;

    for d in 1..=max_depth {
        let (score, best_move) = unsafe { search_fixed_depth(board, d, env) };

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
                let pv_str = env.format_pv();

                println!(
                    "info depth {} score {} nodes {} pv {}",
                    d,
                    score_str,
                    env.nodes_visited,
                    if pv_str.is_empty() {
                        format!("{}", mv)
                    } else {
                        pv_str
                    }
                );
            }
        }
    }

    (global_best_score, global_best_move)
}
