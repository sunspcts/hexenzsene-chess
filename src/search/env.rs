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
    pub fn new(
        tt: &'a mut TT,
        hash_history: Vec<u64>,
        search_control: SearchControl,
        node_limit: u64,
        age: u8,
    ) -> Self {
        Self {
            nodes_visited: 0,
            node_limit,
            silent: false,
            hash_history,
            search_control,
            stopped: false,
            age,
            tt,
            killers: KillerTable::new(),
            history: HistoryTable::new(),
            move_lists: [MoveList::default(); MAX_PLY],
            pv: PvTable::new(),
        }
    }

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
    pub fn new(alpha: i64, beta: i64, depth: i64, lmr_allowed: bool) -> Self {
        SearchContext {
            alpha,
            beta,
            ply: 0,
            depth,
            is_pv: true,
            lmr_allowed,
        }
    }

    #[allow(dead_code)]
    pub fn new_full_window(depth: i64, lmr_allowed: bool) -> Self {
        Self::new(-1_000_000, 1_000_000, depth, lmr_allowed)
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
