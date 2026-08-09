mod psts;
mod pawn_structure;
mod mobility;

use psts::*;
use pawn_structure::*;
use mobility::*;

use crate::board::Board;

//useful to allow tuning routines to fuck with the params.
#[derive(Clone, Copy)]
pub struct EvalParams {
    pub isolated_pawn_mg: i64,
    pub isolated_pawn_eg: i64,
    pub passed_pawn_mg: [i64; 8],
    pub passed_pawn_eg: [i64; 8],
    pub doubled_pawn_mg: [i64; 4],
    pub doubled_pawn_eg: [i64; 4],
    pub mg_piece_values: [i64; 6],
    pub eg_piece_values: [i64; 6],
    pub mg_psts: [[i64; 64]; 6],
    pub eg_psts: [[i64; 64]; 6],
    pub knight_mobility_mg: [i64; 9],
    pub knight_mobility_eg: [i64; 9],

}

impl Default for EvalParams {
    fn default() -> Self {
        Self {
            isolated_pawn_mg: ISOLATED_PAWN_MG,
            isolated_pawn_eg: ISOLATED_PAWN_EG,
            passed_pawn_mg: PASSED_PAWN_MG,
            passed_pawn_eg: PASSED_PAWN_EG,
            doubled_pawn_mg: DOUBLED_PAWN_MG,
            doubled_pawn_eg: DOUBLED_PAWN_EG,
            mg_piece_values: MG_PIECE_VALUES,
            eg_piece_values: EG_PIECE_VALUES,
            mg_psts: MG_PSTS,
            eg_psts: EG_PSTS,
            knight_mobility_mg: KNIGHT_MOBILITY_MG,
            knight_mobility_eg: KNIGHT_MOBILITY_EG,
        }
    }
}

pub fn eval(board: &Board) -> i64 {
    eval_with_params(board, &EvalParams::default())
}

#[inline]
pub fn eval_with_params(board: &Board, params: &EvalParams) -> i64 {
    // Game phase is calculated as (QUEENS * 4 + ROOKS * 2 + BISHOPS + KNIGHTS)
    let mut phase = 0;
    for color in 0..2 {
        for (piece, bb) in board.piece_bb[color].iter().enumerate() {
            phase += PIECE_PHASE[piece] * bb.count_ones() as i64;
        }
    }

    phase = phase.min(MAX_PHASE);

    let mut score = 0;

    pawn_eval(board, &mut score, phase, params);
    knight_eval(board, &mut score, phase, params);

    for p in 2..6 {
        standard_eval(p, board, &mut score, phase, params);
    }

    score * board.side_to_move_multiplier()
}

#[inline]
fn pawn_eval(board: &Board, score: &mut i64, phase: i64, params: &EvalParams) {
    let white = board.piece_bb[0][0]; let black = board.piece_bb[1][0];
    let eg_phase = MAX_PHASE - phase;
    *score += calc_tapered_score_with_params(0, phase, white, 56, &params.mg_piece_values, &params.eg_piece_values, &params.mg_psts, &params.eg_psts);
    *score -= calc_tapered_score_with_params(0, phase, black, 0, &params.mg_piece_values, &params.eg_piece_values, &params.mg_psts, &params.eg_psts);

    for sq in white {
        if is_isolated(sq, white) {
            *score += (params.isolated_pawn_mg * phase + params.isolated_pawn_eg * eg_phase) / MAX_PHASE;
        }
        if is_passed(sq, 0, black) {
            let rank = (7 - (sq / 8)) as usize;
            *score += (params.passed_pawn_mg[rank] * phase + params.passed_pawn_eg[rank] * eg_phase) / MAX_PHASE;
        }
    }
    for sq in black {
        if is_isolated(sq, black) {
            *score -= (params.isolated_pawn_mg * phase + params.isolated_pawn_eg * eg_phase) / MAX_PHASE;
        }
        if is_passed(sq, 1, white) {
            let rank = (sq / 8) as usize;
            *score -= (params.passed_pawn_mg[rank] * phase + params.passed_pawn_eg[rank] * eg_phase) / MAX_PHASE;
        }
    }

    for f in 0..8 {
        let file_mask = FILE_MASKS[f];
        let white_count = (white & file_mask).count_ones() as i64;

        let group = if f < 4 { f } else { 7 - f };

        if white_count > 1 {
            let count = white_count - 1;
            *score += count * (params.doubled_pawn_mg[group] * phase + params.doubled_pawn_eg[group] * eg_phase) / MAX_PHASE;
        }

        let black_count = (black & file_mask).count_ones() as i64;
        if black_count > 1 {
            let count = black_count - 1;
            *score -= count * (params.doubled_pawn_mg[group] * phase + params.doubled_pawn_eg[group] * eg_phase) / MAX_PHASE;
        }
    }
}

fn knight_eval(board: &Board, score: &mut i64, phase: i64, params: &EvalParams) {
    let white = board.piece_bb[0][1]; let black = board.piece_bb[1][1];

    *score += calc_tapered_score_with_params(1, phase, white, 56, &params.mg_piece_values, &params.eg_piece_values, &params.mg_psts, &params.eg_psts);
    *score -= calc_tapered_score_with_params(1, phase, black, 0, &params.mg_piece_values, &params.eg_piece_values, &params.mg_psts, &params.eg_psts);

    let mg_mob = knight_mobility_score(board, &params.knight_mobility_mg);
    let eg_mob = knight_mobility_score(board, &params.knight_mobility_eg);

    *score += (mg_mob * phase + eg_mob * (MAX_PHASE - phase)) / MAX_PHASE;
}

fn standard_eval(piece: usize, board: &Board, score: &mut i64, phase: i64, params: &EvalParams) {
    let white = board.piece_bb[0][piece]; let black = board.piece_bb[1][piece];

    *score += calc_tapered_score_with_params(piece, phase, white, 56, &params.mg_piece_values, &params.eg_piece_values, &params.mg_psts, &params.eg_psts);
    *score -= calc_tapered_score_with_params(piece, phase, black, 0, &params.mg_piece_values, &params.eg_piece_values, &params.mg_psts, &params.eg_psts);
}