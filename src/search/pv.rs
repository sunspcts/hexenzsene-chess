use crate::{moves::Move, search::MAX_PLY};

#[derive(Clone)]
pub struct PvTable {
    table: [[Move; MAX_PLY]; MAX_PLY],
    len: [usize; MAX_PLY],
}

impl Default for PvTable {
    fn default() -> Self {
        Self::new()
    }
}

impl PvTable {
    pub fn new() -> Self {
        Self {
            table: [[Move::new_from_raw(0); MAX_PLY]; MAX_PLY],
            len: [0; MAX_PLY],
        }
    }

    #[inline(always)]
    pub fn clear_ply(&mut self, ply: usize) {
        let p = ply.min(MAX_PLY - 1);
        self.len[p] = 0;
        self.table[p][0] = Move::new_from_raw(0);
    }

    #[inline]
    pub fn update(&mut self, ply: usize, mv: Move) {
        let next_ply = (ply + 1).min(MAX_PLY - 1);
        let max_child = MAX_PLY.saturating_sub(ply + 2);
        let child_len = self.len[next_ply].min(max_child);
        self.table[ply][0] = mv;
        for j in 0..child_len {
            self.table[ply][1 + j] = self.table[next_ply][j];
        }
        let total_len = 1 + child_len;
        if total_len < MAX_PLY {
            self.table[ply][total_len] = Move::new_from_raw(0);
        }
        self.len[ply] = total_len;
    }

    #[inline]
    pub fn pv_move(&self, ply: usize) -> Option<Move> {
        if ply < self.len[0] && self.table[0][ply].data() != 0 {
            Some(self.table[0][ply])
        } else {
            None
        }
    }

    #[inline]
    pub fn root_move(&self) -> Option<Move> {
        self.pv_move(0)
    }

    pub fn format_pv(&self) -> String {
        let len = self.len[0];
        let mut pv_str = String::new();
        for i in 0..len {
            let mv = self.table[0][i];
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pv_table_update_and_format() {
        let mut pv = PvTable::new();
        let m1 = Move::new_from_raw(1);
        let m2 = Move::new_from_raw(2);

        pv.clear_ply(1);
        pv.update(1, m2);

        pv.clear_ply(0);
        pv.update(0, m1);

        assert_eq!(pv.pv_move(0), Some(m1));
        assert_eq!(pv.pv_move(1), Some(m2));
        assert_eq!(pv.pv_move(2), None);
    }
}
