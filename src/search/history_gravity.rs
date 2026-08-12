use crate::moves::Move;

// History values are naturally clamped to 5000.
pub const MAX_HISTORY: i32 = 5000;

#[inline]
pub fn update_history_cutoff(
    history: &mut [[[i32; 64]; 64]; 2],
    side: usize,
    cutoff_move: Move,
    quiet_moves_tried: &[Move],
    depth: i64,
) {
    let delta = (depth * depth) as i32;

    // Bonus for cutoff move
    let from = cutoff_move.from_sq() as usize;
    let to = cutoff_move.to_sq() as usize;
    let current_val = history[side][from][to];

    let bonus = delta - (current_val * delta) / MAX_HISTORY; // If current_val == MAX_HISTORY, bonus = 0
    history[side][from][to] += bonus;

    // Malus for quiet moves that failed to cause a cutoff
    for &quiet_move in quiet_moves_tried {
        let quiet_from = quiet_move.from_sq() as usize;
        let quiet_to = quiet_move.to_sq() as usize;
        let quiet_val = history[side][quiet_from][quiet_to];
        let malus = -delta - (quiet_val * delta) / MAX_HISTORY;
        history[side][quiet_from][quiet_to] += malus;
    }
}
