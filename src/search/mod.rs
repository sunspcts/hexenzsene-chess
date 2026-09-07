mod format;
mod history_gravity;
mod killer_heuristic;
mod lmr;
mod negamax;
mod ordering;
mod pv;
mod qsearch;
mod searcher;
mod tt;

// Private Imports
use format::format_score;
use tt::NodeType;

// Public re-exports
pub use history_gravity::HistoryTable;
pub use pv::PvTable;
pub use searcher::{SearchContext, SearchControl, Searcher};
pub use tt::TT;

use crate::{board::Board, moves::Move};

pub const MATE_EVAL: i64 = 30000;
const NODE_CHECK_INTERVAL_MASK: u64 = 2047; // Check search control every 2048 nodes
pub const MAX_PLY: usize = 256;

impl<'a> Searcher<'a> {
    fn search_fixed_depth(
        &mut self,
        board: &Board,
        depth: i64,
        alpha: i64,
        beta: i64,
    ) -> (i64, Option<Move>) {
        const PLY: usize = 0;
        // Fetch the TT move.
        let tt_move = self
            .tt
            .get(board.game_state.curr_zobrist_key)
            .and_then(|e| e.best_move());

        let in_check = board.is_in_check();
        let root_depth = depth + in_check as i64;

        let mut context = SearchContext::new(alpha, beta, root_depth, root_depth >= 3 && !in_check);
        let old_alpha = context.alpha;

        self.generate_and_order_moves(board, tt_move, PLY, context.is_pv);
        self.pv.clear_ply(0);
        let mut movedata = self.iterate_moves(board, &mut context, root_depth, PLY);

        if movedata.max_score == i64::MIN {
            movedata.max_score = context.alpha;
        }

        self.store_tt_entry(board, &movedata, &context, old_alpha, depth);

        (movedata.max_score, movedata.best_move)
    }

    fn search_aspiration(
        &mut self,
        board: &Board,
        depth: i64,
        prev_score: i64,
    ) -> (i64, Option<Move>) {
        if depth < 4 {
            return self.search_fixed_depth(board, depth, -1_000_000, 1_000_000);
        }

        let mut delta = 35;
        let mut alpha = (prev_score - delta).max(-1_000_000);
        let mut beta = (prev_score + delta).min(1_000_000);

        loop {
            let (score, best_move) = self.search_fixed_depth(board, depth, alpha, beta);
            if self.stopped {
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

    // Handler. Performs iterative deepening search up to the specified maximum depth.
    pub fn search(&mut self, board: &Board, max_depth: i64) -> (i64, Option<Move>) {
        let mut global_best_move = None;
        let mut global_best_score = 0;

        for d in 1..=max_depth {
            let (score, best_move) = self.search_aspiration(board, d, global_best_score);

            if self.stopped {
                if global_best_move.is_none() && best_move.is_some() {
                    global_best_move = best_move;
                    global_best_score = score;
                }
                break;
            }

            if let Some(mv) = best_move {
                global_best_move = Some(mv);
                global_best_score = score;

                if !self.silent {
                    let score_str = format_score(score);
                    let pv_str = self.pv.format_pv();
                    let pv = if pv_str.is_empty() {
                        format!("{mv}")
                    } else {
                        pv_str
                    };
                    println!(
                        "info depth {d} score {score_str} nodes {} pv {pv}",
                        self.nodes_visited
                    );
                }
            }
        }

        (global_best_score, global_best_move)
    }
}
