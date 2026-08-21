use crate::piece::Piece;

// The table is 7x7 rather than 6x6 because score_qsearch_moves checks it unconditionally.
const MVV_LVA_TAB: [[i16; 7]; 7] = init_mvv_lva_table();

const fn init_mvv_lva_table() -> [[i16; 7]; 7] {
    let mut tab = [[0; 7]; 7];
    let mut a = 0;
    while a < 6 {
        let mut v = 0;
        while v < 6 {
            tab[v][a] = (((v + 1) * 10) + (6 - a)) as i16;
            v += 1;
        }
        a += 1;
    }
    tab
}

#[inline]
pub fn calc_mvv_lva_heuristic(piece: Piece, enemy_piece: Piece) -> i16 {
    MVV_LVA_TAB[enemy_piece as usize][piece as usize]
}
