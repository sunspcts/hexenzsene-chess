use super::MATE_EVAL;
use crate::moves::Move;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum NodeType {
    #[default]
    None = 0,
    Exact = 1,
    LowerBound = 2,
    UpperBound = 3,
}

#[derive(Clone, Copy, Default)]
pub struct TTEntry {
    pub zobrist_key: u64,
    pub score: i16,
    pub move_data: u16,
    pub depth: i8,
    pub node_type: NodeType,
    pub age: u8,
}

#[inline]
pub fn score_to_tt(score: i64, ply: i64) -> i16 {
    if score > MATE_EVAL - 1000 {
        (score + ply) as i16
    } else if score < -MATE_EVAL + 1000 {
        (score - ply) as i16
    } else {
        score as i16
    }
}

#[inline]
fn score_from_tt(score: i16, ply: i64) -> i64 {
    let s = score as i64;
    if s > MATE_EVAL - 1000 {
        s - ply
    } else if s < -MATE_EVAL + 1000 {
        s + ply
    } else {
        s
    }
}

impl TTEntry {
    pub fn best_move(&self) -> Option<Move> {
        if self.move_data == 0 {
            None
        } else {
            Some(Move::new_from_raw(self.move_data))
        }
    }

    pub fn cutoff(&self, alpha: i64, beta: i64, depth: i64, ply: i64) -> Option<i64> {
        if (self.depth as i64) >= depth {
            let score = score_from_tt(self.score, ply);
            match self.node_type {
                NodeType::Exact => Some(score),
                NodeType::LowerBound if score >= beta => Some(score),
                NodeType::UpperBound if score <= alpha => Some(score),
                _ => None,
            }
        } else {
            None
        }
    }
}

pub struct TT {
    entries: Vec<TTEntry>,
    mask: usize,
}

impl TT {
    pub fn new(size_mb: usize) -> Self {
        let bytes = (size_mb).max(1) * 1024 * 1024;
        let target_entries = bytes / std::mem::size_of::<TTEntry>();
        let num_entries = 1usize << (usize::BITS - 1 - target_entries.leading_zeros());
        TT {
            entries: vec![TTEntry::default(); num_entries],
            mask: num_entries - 1,
        }
    }

    pub fn clear(&mut self) {
        self.entries.fill(TTEntry::default());
    }

    #[inline(always)]
    pub fn get(&self, zobrist_key: u64) -> Option<TTEntry> {
        let index = (zobrist_key as usize) & self.mask;
        let entry = unsafe { *self.entries.get_unchecked(index) };
        if entry.node_type != NodeType::None && entry.zobrist_key == zobrist_key {
            Some(entry)
        } else {
            None
        }
    }

    pub fn store(&mut self, entry: TTEntry) {
        let index = (entry.zobrist_key as usize) & self.mask;
        let existing = unsafe { *self.entries.get_unchecked(index) };
        let mut entry = entry;
        if existing.node_type != NodeType::None {
            let is_stale = existing.age != entry.age;
            if !is_stale {
                let existing_is_pv = existing.node_type == NodeType::Exact;
                let entry_is_pv = entry.node_type == NodeType::Exact;

                let existing_is_better = if existing_is_pv && !entry_is_pv {
                    existing.depth >= entry.depth
                } else {
                    existing.depth > entry.depth
                };

                if existing_is_better {
                    if existing.zobrist_key == entry.zobrist_key
                        && existing.move_data == 0
                        && entry.move_data != 0
                    {
                        unsafe {
                            self.entries.get_unchecked_mut(index).move_data = entry.move_data;
                        }
                    }
                    return;
                }
            }
            if existing.zobrist_key == entry.zobrist_key
                && entry.move_data == 0
                && existing.move_data != 0
            {
                entry.move_data = existing.move_data;
            }
        }
        unsafe {
            *self.entries.get_unchecked_mut(index) = entry;
        }
    }
}
