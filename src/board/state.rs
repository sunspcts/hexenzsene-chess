use super::Side;

// STATE STRUCT

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameState {
    pub active_side: Side,
    pub castling: u8,
    pub half_moves: u8,
    pub move_counter: u16,
    pub en_passant_square: Option<u8>, //unfortunately, it's unprofessional to call this the holy_hell_square.
    pub curr_zobrist_key: u64,
}

impl GameState {
    pub fn inc_halfmoves(&mut self) {
        self.half_moves += 1
    }

    pub fn reset_halfmoves(&mut self) {
        self.half_moves = 0
    }

    pub fn inc_count(&mut self) {
        self.move_counter += 1
    }
}
