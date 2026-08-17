use crate::{board::Board};

use super::Move;

// UCI format.
impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from_file = (b'a' + (self.from_sq() % 8) as u8) as char;
        let from_rank = (b'1' + (self.from_sq() / 8) as u8) as char;
        let to_file = (b'a' + (self.to_sq() % 8) as u8) as char;
        let to_rank = (b'1' + (self.to_sq() / 8) as u8) as char;

        if self.is_promo() {
            let promo_char = match self.flags() & 0b0011 {
                0 => 'n',
                1 => 'b',
                2 => 'r',
                3 => 'q',
                _ => unreachable!(),
            };
            write!(f, "{from_file}{from_rank}{to_file}{to_rank}{promo_char}")?;
        } else {
            write!(f, "{from_file}{from_rank}{to_file}{to_rank}")?;
        }

        Ok(())
    }
}

impl Move {
    // Generates all possible moves, checks if the uci_string passed matches any of them. Returns None as a fallback.
    pub fn from_uci(board: &Board, uci_str: &str) -> Option<Move> {
        let mut moves = super::MoveList::default();
        moves.generate_pseudolegal_moves(board);
        moves
            .into_iter()
            .find(|&mv| mv.to_string() == uci_str && board.make(mv).is_some())
    }
}
