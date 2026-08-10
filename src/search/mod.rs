mod negamax;
mod qsearch;
mod tt;
mod env;
mod format;
mod nmp;
mod history_gravity;

use negamax::negamax;

use qsearch::quiescense;
pub use tt::*;
pub use env::*;
use format::*;

use crate::{board::Board, moves::Move};

pub const MATE_EVAL: i64 = 30000;
const NODE_CHECK_INTERVAL_MASK: u64 = 2047; // Check search control every 2048 nodes
pub const MAX_PLY: usize = 256;

fn search_fixed_depth(board: &Board, depth: i64, env: &mut SearchEnv) -> (i64, Option<Move>) {
    let tt_move = env.tt.get(board.game_state.curr_zobrist_key).and_then(|e| e.best_move());
    let ply = 0;
    let pv_move = if env.pv_length[0] > 0 && env.pv_table[0][0].data() != 0 {
        Some(env.pv_table[0][0])
    } else {
        None
    };

    env.pv_length[0] = 0;
    env.pv_table[0][0] = Move::new_from_raw(0);

    board.generate_pseudolegal_moves(&mut env.move_lists[ply]);
    env.move_lists[ply].score_moves(board, pv_move, tt_move, &env.killers[ply], &env.history);
    let moves_count = env.move_lists[ply].len();

    let mut best_move = None;
    let mut max_score = i64::MIN;
    let mut alpha = -1_000_000;
    let beta = 1_000_000;
    let mut legal_moves_count = 0;

    for i in 0..moves_count {
        let candidate_move = env.move_lists[ply].pick_best(i);
        if let Some(next_board) = board.make(candidate_move) {
            legal_moves_count += 1;

            env.hash_history.push(board.game_state.curr_zobrist_key);

            let score = if legal_moves_count == 1 {
                let context = SearchContext {
                    alpha: -beta,
                    beta: -alpha,
                    depth: depth - 1,
                    ply: 1,
                    is_pv: true,
                    nmp_allowed: true,
                };
                -negamax(&next_board, context, env)
            } else {
                let null_context = SearchContext {
                    alpha: -alpha - 1,
                    beta: -alpha,
                    depth: depth - 1,
                    ply: 1,
                    is_pv: false,
                    nmp_allowed: true,
                };
                let mut s = -negamax(&next_board, null_context, env);
                if s > alpha && s < beta {
                    let full_context = SearchContext {
                        alpha: -beta,
                        beta: -alpha,
                        depth: depth - 1,
                        ply: 1,
                        is_pv: true,
                        nmp_allowed: true,
                    };
                    s = -negamax(&next_board, full_context, env);
                }
                s
            };

            env.hash_history.pop();

            if env.stopped {
                break;
            }

            if score > max_score {
                max_score = score;
                best_move = Some(candidate_move);
            }

            if score > alpha {
                alpha = score;

                let child_len = env.pv_length[1];
                env.pv_table[0][0] = candidate_move;
                for j in 0..child_len {
                    env.pv_table[0][1 + j] = env.pv_table[1][j];
                }
                if 1 + child_len < MAX_PLY {
                    env.pv_table[0][1 + child_len] = Move::new_from_raw(0);
                }
                env.pv_length[0] = 1 + child_len;
            }
        }
    }

    if !env.stopped && best_move.is_some() {
        env.tt.store(TTEntry {
            zobrist_key: board.game_state.curr_zobrist_key,
            score: score_to_tt(max_score, 0),
            move_data: best_move.map(|m| m.data()).unwrap_or(0),
            depth: depth as i8,
            node_type: NodeType::Exact,
            age: env.age,
        });
    }

    (max_score, best_move)
}

pub fn search(board: &Board, max_depth: i64, env: &mut SearchEnv) -> (i64, Option<Move>) {
    let mut global_best_move = None;
    let mut global_best_score = 0;

    for d in 1..=max_depth {
        let (score, best_move) = search_fixed_depth(board, d, env);

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

            let score_str = format_score(score);
            let pv_str = env.format_pv();

            println!(
                "info depth {} score {} nodes {} pv {}",
                d, score_str, env.nodes_visited, if pv_str.is_empty() { format!("{}", mv) } else { pv_str }
            );
        }
    }

    (global_best_score, global_best_move)
}