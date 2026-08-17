use crate::moves::Move;

// History values are naturally clamped to MAX_HISTORY.
pub const MAX_HISTORY: i32 = 5000;

pub struct HistoryTable {
    table: [[[i32; 64]; 64]; 2],
}

impl HistoryTable {
    pub fn new() -> Self {
        Self {
            table: [[[0; 64]; 64]; 2],
        }
    }

    #[inline]
    pub fn update_cutoff(
        &mut self,
        side: usize,
        cutoff_move: Move,
        quiet_moves_tried: &[Move],
        depth: i64,
    ) {
        let delta = (depth * depth) as i32;

        // Bonus for cutoff move
        let from = cutoff_move.from_sq() as usize;
        let to = cutoff_move.to_sq() as usize;
        let current_val = self.table[side][from][to];

        let bonus = delta - (current_val * delta) / MAX_HISTORY; // If current_val == MAX_HISTORY, bonus = 0
        self.table[side][from][to] += bonus;

        // Malus for quiet moves that failed to cause a cutoff
        for &quiet_move in quiet_moves_tried {
            let quiet_from = quiet_move.from_sq() as usize;
            let quiet_to = quiet_move.to_sq() as usize;
            let quiet_val = self.table[side][quiet_from][quiet_to];
            let malus = -delta - (quiet_val * delta) / MAX_HISTORY;
            self.table[side][quiet_from][quiet_to] += malus;
        }
    }

    #[inline(always)]
    pub fn get(&self, side: usize, mv: Move) -> i32 {
        self.table[side][mv.from_sq() as usize][mv.to_sq() as usize]
    }
}
