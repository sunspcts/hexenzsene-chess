use crate::{attacks::KNIGHT_ATTACKS, bitboard::Bitboard, board::Board, piece::Piece};

pub const KNIGHT_MOBILITY: [i64; 9] = [-20, -15, -10, -5, 0, 5, 10, 15, 20]; // Arbitrary, the tuner will figure these out.

const NOT_A_FILE: Bitboard = Bitboard::new(!0x0101010101010101);
const NOT_H_FILE: Bitboard = Bitboard::new(!0x8080808080808080);

#[inline]
pub fn knight_mobility_score(board: &Board, mobility_table: &[i64; 9]) -> i64 {
    let mut score = 0;

    let black_pawns = board.piece_bb[1][Piece::Pawn as usize];
    let black_pawn_attacks = ((black_pawns & NOT_H_FILE) >> 7) | ((black_pawns & NOT_A_FILE) >> 9);

    let white_pawns = board.piece_bb[0][Piece::Pawn as usize];
    let white_pawn_attacks = ((white_pawns & NOT_A_FILE) << 7) | ((white_pawns & NOT_H_FILE) << 9);

    let white_knights = board.piece_bb[0][Piece::Knight as usize];
    for sq in white_knights {
        let moves = KNIGHT_ATTACKS[sq as usize] & !board.side_bb[0] & !black_pawn_attacks;
        let count = moves.count_ones() as usize;
        score += mobility_table[count];
    }

    let black_knights = board.piece_bb[1][Piece::Knight as usize];
    for sq in black_knights {
        let moves = KNIGHT_ATTACKS[sq as usize] & !board.side_bb[1] & !white_pawn_attacks;
        let count = moves.count_ones() as usize;
        score -= mobility_table[count];
    }

    score
}