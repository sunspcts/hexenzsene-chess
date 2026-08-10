use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use super::*;

use crate::moves::MoveList;

// Holds global search variables shared across the recursion
pub struct SearchEnv<'a> {
    pub nodes_visited: u64,
    pub node_limit: u64,
    pub hash_history: Vec<u64>,
    pub search_control: SearchControl,
    pub stopped: bool,
    pub age: u8,
    pub tt: &'a mut TT,
    pub killers: [[u16; 2]; MAX_PLY],
    pub history: [[[i32; 64]; 64]; 2],
    pub move_lists: [MoveList; MAX_PLY],
    pub pv_table: [[Move; MAX_PLY]; MAX_PLY],
    pub pv_length: [usize; MAX_PLY],
}

impl<'a> SearchEnv<'a> {
    pub fn format_pv(&self) -> String {
        let len = self.pv_length[0];
        let mut pv_str = String::new();
        for i in 0..len {
            let mv = self.pv_table[0][i];
            if mv.data() == 0 {
                break;
            }
            if i > 0 {
                pv_str.push(' ');
            }
            pv_str.push_str(&format!("{}", mv));
        }
        pv_str
    }

    #[inline(always)]
    pub fn is_repetition(&self, key: u64, half_moves: usize) -> bool {
        self.hash_history.iter().rev().take(half_moves).any(|&k| k == key)
    }

    #[inline(always)]
    pub fn step_node_and_check(&mut self) -> bool {
        self.nodes_visited += 1;
        if self.stopped || self.nodes_visited >= self.node_limit {
            self.stopped = true;
            return true;
        }
        if (self.nodes_visited & NODE_CHECK_INTERVAL_MASK == 0) && self.search_control.is_stopped() {
            self.stopped = true;
            return true;
        }
        false
    }
}

#[derive(Clone, Copy)]
pub(super) struct SearchContext {
    pub alpha: i64,
    pub beta: i64,
    pub ply: i64,
    pub depth: i64,
    pub is_pv: bool,
    #[allow(dead_code)]
    pub nmp_allowed: bool,
}

impl SearchContext {
    pub fn next_context(&self, depth: i64, is_pv: bool) -> Self {
        SearchContext {
            alpha: -self.beta,
            beta: -self.alpha,
            ply: self.ply + 1,
            depth,
            is_pv,
            nmp_allowed: true
        }
    }

    #[inline]
    pub fn update_alpha(&mut self, score: i64) -> bool {
        if score > self.alpha {
            self.alpha = score;
        }
        self.alpha >= self.beta
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

    pub fn next_context_null_window(&self, depth: i64) -> Self {
        SearchContext {
            alpha: -self.alpha - 1,
            beta: -self.alpha,
            ply: self.ply + 1,
            depth,
            is_pv: false,
            nmp_allowed: true
        }
    }
}

#[derive(Clone)]
pub struct SearchControl {
    pub stop: Arc<AtomicBool>,
}

impl SearchControl {
    pub fn new() -> Self {
        SearchControl { stop: Arc::new(AtomicBool::new(false)) }
    }

    pub fn is_stopped(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
