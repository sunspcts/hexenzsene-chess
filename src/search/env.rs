use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use super::*;

use crate::{
    moves::MoveList,
    search::{history_gravity::HistoryTable, killer_heuristic::KillerTable},
};

// Holds global search variables. Initialized at the start of each search. (Gonna create a nice ::new() function at some point to avoid having all public fields.)
pub struct SearchEnv<'a> {
    pub nodes_visited: u64,            // TOTAL nodes visited across the search.
    pub node_limit: u64,               // Node count at which the search returns early.
    pub silent: bool, // When true, suppresses stdout info output (used for benchmarks).
    pub hash_history: Vec<u64>, // History of zobrist hashes, used for repetition detection.
    pub search_control: SearchControl, // Wrapper around an Arc<AtomicBool>, so the engine thread can stop the search thread. Polled occasionally.
    pub stopped: bool,
    pub age: u8, // increments every time a search is run, essentially a move counter in normal practice ("Which move of the game was this search run on?").
    pub tt: &'a mut TT, // Mutable reference to the Transposition Table.
    pub killers: KillerTable, // Killer Move Storage (Ordering)
    pub history: HistoryTable, // History Table Storage (Ordering)
    pub move_lists: [MoveList; MAX_PLY], // One move list per ply.
    pub pv: PvTable, // Triangular PV table.
}

impl<'a> SearchEnv<'a> {
    #[inline(always)]
    pub fn is_repetition(&self, key: u64, half_moves: usize) -> bool {
        // Only looks back until the last move which reset the HMC.
        self.hash_history
            .iter()
            .rev()
            .take(half_moves)
            .any(|&k| k == key)
    }

    #[inline(always)]
    pub fn step_node_and_check(&mut self) -> bool {
        // Increases node counter, checks stopped bool. Checks in with the control thread every NODE_CHECK_INTERVAL_MASK nodes.
        self.nodes_visited += 1;
        if self.stopped || self.nodes_visited >= self.node_limit {
            self.stopped = true;
            return true;
        }
        if (self.nodes_visited & NODE_CHECK_INTERVAL_MASK == 0) && self.search_control.is_stopped()
        {
            self.stopped = true;
            return true;
        }
        false
    }

    #[inline]
    pub fn is_draw(&self, board: &Board, ply: usize) -> bool {
        ply > 0
            && (board.game_state.half_moves >= 100
                || self.is_repetition(
                    board.game_state.curr_zobrist_key,
                    board.game_state.half_moves as usize,
                ))
    }
}

#[derive(Clone, Copy)]
pub struct SearchContext {
    pub alpha: i64,
    pub beta: i64,
    ply: usize,
    pub depth: i64,
    pub is_pv: bool,
    pub lmr_allowed: bool,
}

impl SearchContext {
    pub fn new_full_window(depth: i64, lmr_allowed: bool) -> Self {
        SearchContext {
            alpha: -1_000_000,
            beta: 1_000_000,
            ply: 0,
            depth: depth,
            is_pv: true,
            lmr_allowed,
        }
    }

    pub fn ply(&self) -> usize {
        self.ply
    }
    #[inline(always)]
    pub fn next_context(&self, depth: i64, is_pv: bool) -> Self {
        SearchContext {
            alpha: -self.beta,
            beta: -self.alpha,
            ply: self.ply + 1,
            depth,
            is_pv,
            lmr_allowed: self.lmr_allowed,
        }
    }

    #[inline]
    pub fn node_type(&self, max_score: i64, old_alpha: i64) -> NodeType {
        if max_score >= self.beta {
            NodeType::LowerBound
        } else if max_score > old_alpha {
            NodeType::Exact
        } else {
            NodeType::UpperBound
        }
    }

    #[inline(always)]
    pub fn next_context_null_window(&self, depth: i64) -> Self {
        SearchContext {
            alpha: -self.alpha - 1,
            beta: -self.alpha,
            ply: self.ply + 1,
            depth,
            is_pv: false,
            lmr_allowed: self.lmr_allowed,
        }
    }

    #[inline(always)]
    pub fn search_move(
        &self,
        board: &Board,
        depth: i64,
        move_count: usize,
        is_quiet: bool,
        is_killer: bool,
        env: &mut SearchEnv,
    ) -> i64 {
        let is_first_move = move_count == 0;
        if is_first_move {
            -negamax(board, self.next_context(depth, self.is_pv), env)
        } else {
            let can_reduce = self.lmr_allowed && is_quiet && !is_killer && move_count >= 3;

            let mut score = if can_reduce {
                let depth_clamp = (self.depth as usize).min(63);
                let move_clamp = move_count.min(63);

                let reduction = LM_REDUCTIONS_TABLE[depth_clamp][move_clamp];
                let lmr_score =
                    -negamax(board, self.next_context_null_window(depth - reduction), env); // Search at reduced depth with a null window.

                if lmr_score > self.alpha {
                    // We failed high! Research at full depth.
                    -negamax(board, self.next_context_null_window(depth), env)
                } else {
                    lmr_score
                }
            } else {
                -negamax(board, self.next_context_null_window(depth), env)
            };

            if self.is_pv && score > self.alpha && score < self.beta {
                // Failed high even at full depth. Let's run with a full window to get an accurate score.
                score = -negamax(board, self.next_context(depth, self.is_pv), env);
            }
            score
        }
    }
}

#[derive(Clone)]
pub struct SearchControl {
    pub stop: Arc<AtomicBool>,
}

impl SearchControl {
    pub fn new() -> Self {
        SearchControl {
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
