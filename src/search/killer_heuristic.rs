use crate::{moves::Move, search::MAX_PLY};

#[derive(Clone, Debug)]
pub struct KillerTable {
    killers: [[Move; 2]; MAX_PLY],
}

impl Default for KillerTable {
    fn default() -> Self {
        Self::new()
    }
}

impl KillerTable {
    pub fn new() -> Self {
        Self {
            killers: [[Move::new_from_raw(0); 2]; MAX_PLY],
        }
    }

    #[inline(always)]
    pub fn add(&mut self, ply: usize, mv: Move) {
        if ply < MAX_PLY && self.killers[ply][0] != mv {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = mv;
        }
    }

    #[inline(always)]
    pub fn get(&self, ply: usize) -> [Move; 2] {
        if ply < MAX_PLY {
            self.killers[ply]
        } else {
            [Move::new_from_raw(0); 2]
        }
    }

    #[inline(always)]
    pub fn is_killer(&self, ply: usize, mv: Move) -> bool {
        if ply < MAX_PLY {
            self.killers[ply][0] == mv || self.killers[ply][1] == mv
        } else {
            false
        }
    }
}