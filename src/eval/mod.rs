mod psts;
mod pawn_structure;

use psts::*;
use pawn_structure::*;
use crate::{board::Board, piece::Piece};

const ISOLATED_PAWN_MG: i64 = -17;
const ISOLATED_PAWN_EG: i64 = -10;

const PASSED_PAWN_MG: [i64; 8] = [0, 0, 25, 0, 8, -7, 23, 0];
const PASSED_PAWN_EG: [i64; 8] = [0, 0, 47, 47, 31, 55, 22, 0];

const DOUBLED_PAWN_MG: [i64; 4] = [-11, -9, -6, -4]; // Group 0 (A/H), 1 (B/G), 2 (C/F), 3 (D/E)
const DOUBLED_PAWN_EG: [i64; 4] = [-14, -11, -8, -6];

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
        }
    }
}

pub fn eval(board: &Board) -> i64 {
    eval_with_params(board, &EvalParams::default())
}

#[inline]
pub fn eval_with_params(board: &Board, params: &EvalParams) -> i64 {
    let mut phase = 0;
    for color in 0..2 {
        for (piece, bb) in board.piece_bb[color].iter().enumerate() {
            phase += PIECE_PHASE[piece] * bb.count_ones() as i64;
        }
    }
    let mg_phase = phase.min(MAX_PHASE);
    let eg_phase = MAX_PHASE - mg_phase;

    let mut score = 0;

    for (piece, bb) in board.piece_bb[0].iter().enumerate() {
        score += calc_tapered_score_with_params(piece, mg_phase, *bb, 56, &params.mg_piece_values, &params.eg_piece_values, &params.mg_psts, &params.eg_psts);
    }

    for (piece, bb) in board.piece_bb[1].iter().enumerate() {
        score -= calc_tapered_score_with_params(piece, mg_phase, *bb, 0, &params.mg_piece_values, &params.eg_piece_values, &params.mg_psts, &params.eg_psts);
    }

    // Isolated pawn penalties
    let white_pawns = board.piece_bb[0][Piece::Pawn as usize];
    for sq in white_pawns {
        if is_isolated(sq, white_pawns) {
            score += (params.isolated_pawn_mg * mg_phase + params.isolated_pawn_eg * eg_phase) / MAX_PHASE;
        }
    }

    let black_pawns = board.piece_bb[1][Piece::Pawn as usize];
    for sq in black_pawns {
        if is_isolated(sq, black_pawns) {
            score -= (params.isolated_pawn_mg * mg_phase + params.isolated_pawn_eg * eg_phase) / MAX_PHASE;
        }
    }

    // Passed pawn bonuses
    for sq in white_pawns {
        if is_passed(sq, 0, black_pawns) {
            let rank = (7 - (sq / 8)) as usize;
            score += (params.passed_pawn_mg[rank] * mg_phase + params.passed_pawn_eg[rank] * eg_phase) / MAX_PHASE;
        }
    }

    for sq in black_pawns {
        if is_passed(sq, 1, white_pawns) {
            let rank = (sq / 8) as usize;
            score -= (params.passed_pawn_mg[rank] * mg_phase + params.passed_pawn_eg[rank] * eg_phase) / MAX_PHASE;
        }
    }

    // Doubled pawn penalties (symmetric file groups: 0 = A/H, 1 = B/G, 2 = C/F, 3 = D/E)
    for f in 0..8 {
        let file_mask = FILE_MASKS[f];
        let white_count = (white_pawns & file_mask).count_ones() as i64;
        if white_count > 1 {
            let group = if f < 4 { f } else { 7 - f };
            let count = white_count - 1;
            score += count * (params.doubled_pawn_mg[group] * mg_phase + params.doubled_pawn_eg[group] * eg_phase) / MAX_PHASE;
        }

        let black_count = (black_pawns & file_mask).count_ones() as i64;
        if black_count > 1 {
            let group = if f < 4 { f } else { 7 - f };
            let count = black_count - 1;
            score -= count * (params.doubled_pawn_mg[group] * mg_phase + params.doubled_pawn_eg[group] * eg_phase) / MAX_PHASE;
        }
    }

    score * board.side_to_move_multiplier()
}