use super::*;

use crate::{bitboard::Bitboard, moves::MoveList, piece::Piece};

const A_FILE_BB: Bitboard = Bitboard::new(0x0101010101010101);
const H_FILE_BB: Bitboard = Bitboard::new(0x8080808080808080);
const RANK_3_BB: Bitboard = Bitboard::new(0x0000000000FF0000);
const RANK_6_BB: Bitboard = Bitboard::new(0x0000FF0000000000);
const PROMOTION_RANKS_BB: Bitboard = Bitboard::new(0xFF000000000000FF);

const PROMOTION_FLAGS: [u16; 4] = [
    move_flags::KNIGHT_PROMO,
    move_flags::BISHOP_PROMO,
    move_flags::ROOK_PROMO,
    move_flags::QUEEN_PROMO,
];

impl MoveList {
    pub fn generate_pawn_moves(&mut self, board: &Board) {
        let side = board.game_state.active_side;
        let pawns = board.piece_bb[side as usize][Piece::Pawn as usize];
        let enemy_pieces = board.side_bb[(side as usize) ^ 1]; //this is disgusting but it's kinda a funny way!
        let empty = !(board.side_bb[side as usize] | enemy_pieces);

        let ep_square = board.game_state.en_passant_square;
        let ep_square_bb = Bitboard::new(ep_square.map_or(0, |x| 1u64 << x));

        let attackables = enemy_pieces | ep_square_bb;
        let single_pushes: Bitboard;
        let double_pushes: Bitboard;
        let captures_left: Bitboard;
        let captures_right: Bitboard;

        if side == Side::White {
            single_pushes = (pawns << 8) & empty;
            double_pushes = ((single_pushes & RANK_3_BB) << 8) & empty;
            captures_left = ((pawns & !A_FILE_BB) << 7) & attackables;
            captures_right = ((pawns & !H_FILE_BB) << 9) & attackables;
        } else {
            single_pushes = (pawns >> 8) & empty;
            double_pushes = ((single_pushes & RANK_6_BB) >> 8) & empty;
            captures_left = ((pawns & !A_FILE_BB) >> 9) & attackables;
            captures_right = ((pawns & !H_FILE_BB) >> 7) & attackables;
        }

        let promotion_bb = PROMOTION_RANKS_BB;

        let promo_pushes = single_pushes & promotion_bb;
        let single_pushes = single_pushes & !promotion_bb;

        let promo_caps_left = captures_left & promotion_bb;
        let ep_capture_left = captures_left & ep_square_bb;
        let captures_left = captures_left & !promotion_bb & !ep_square_bb;

        let promo_caps_right = captures_right & promotion_bb;
        let ep_capture_right = captures_right & ep_square_bb;
        let captures_right = captures_right & !promotion_bb & !ep_square_bb;

        let (offset_push, offset_cap_left, offset_cap_right) = if side == Side::White {
            (8, 7, 9)
        } else {
            (-8, -9, -7)
        };

        //Lots of cases!
        pawn_move_helper(single_pushes, offset_push, move_flags::QUIET, false, self);
        pawn_move_helper(
            double_pushes,
            offset_push * 2,
            move_flags::DOUBLE_PAWN_PUSH,
            false,
            self,
        );
        pawn_move_helper(
            captures_left,
            offset_cap_left,
            move_flags::CAPTURE,
            false,
            self,
        );
        pawn_move_helper(
            captures_right,
            offset_cap_right,
            move_flags::CAPTURE,
            false,
            self,
        );
        pawn_move_helper(promo_pushes, offset_push, 0, true, self);
        pawn_move_helper(
            promo_caps_left,
            offset_cap_left,
            move_flags::CAPTURE,
            true,
            self,
        );
        pawn_move_helper(
            promo_caps_right,
            offset_cap_right,
            move_flags::CAPTURE,
            true,
            self,
        );
        pawn_move_helper(
            ep_capture_left,
            offset_cap_left,
            move_flags::EP_CAPTURE,
            false,
            self,
        );
        pawn_move_helper(
            ep_capture_right,
            offset_cap_right,
            move_flags::EP_CAPTURE,
            false,
            self,
        );
    }

    pub fn generate_pawn_caps_promos(&mut self, board: &Board) {
        let side = board.game_state.active_side;
        let pawns = board.piece_bb[side as usize][Piece::Pawn as usize];
        let enemy_pieces = board.side_bb[(side as usize) ^ 1]; //this is disgusting but it's kinda a funny way!
        let empty = !(board.side_bb[side as usize] | enemy_pieces);

        let ep_square = board.game_state.en_passant_square;
        let ep_square_bb = Bitboard::new(ep_square.map_or(0, |x| 1u64 << x));

        let single_pushes = if side == Side::White {
            (pawns << 8) & empty
        } else {
            (pawns >> 8) & empty
        };
        let promo_pushes = single_pushes & PROMOTION_RANKS_BB;

        let attackables = enemy_pieces | ep_square_bb;
        let captures_left: Bitboard;
        let captures_right: Bitboard;

        if side == Side::White {
            captures_left = ((pawns & !A_FILE_BB) << 7) & attackables;
            captures_right = ((pawns & !H_FILE_BB) << 9) & attackables;
        } else {
            captures_left = ((pawns & !A_FILE_BB) >> 9) & attackables;
            captures_right = ((pawns & !H_FILE_BB) >> 7) & attackables;
        }

        let promotion_bb = PROMOTION_RANKS_BB;

        let promo_caps_left = captures_left & promotion_bb;
        let ep_capture_left = captures_left & ep_square_bb;
        let captures_left = captures_left & !promotion_bb & !ep_square_bb;

        let promo_caps_right = captures_right & promotion_bb;
        let ep_capture_right = captures_right & ep_square_bb;
        let captures_right = captures_right & !promotion_bb & !ep_square_bb;

        let (offset_push, offset_cap_left, offset_cap_right) = if side == Side::White {
            (8, 7, 9)
        } else {
            (-8, -9, -7)
        };

        //Lots of cases!
        pawn_move_helper(
            captures_left,
            offset_cap_left,
            move_flags::CAPTURE,
            false,
            self,
        );
        pawn_move_helper(
            captures_right,
            offset_cap_right,
            move_flags::CAPTURE,
            false,
            self,
        );
        pawn_move_helper(promo_pushes, offset_push, 0, true, self);
        pawn_move_helper(
            promo_caps_left,
            offset_cap_left,
            move_flags::CAPTURE,
            true,
            self,
        );
        pawn_move_helper(
            promo_caps_right,
            offset_cap_right,
            move_flags::CAPTURE,
            true,
            self,
        );
        pawn_move_helper(
            ep_capture_left,
            offset_cap_left,
            move_flags::EP_CAPTURE,
            false,
            self,
        );
        pawn_move_helper(
            ep_capture_right,
            offset_cap_right,
            move_flags::EP_CAPTURE,
            false,
            self,
        );
    }
}

#[inline]
fn pawn_move_helper(
    dest_bb: Bitboard,
    offset: i16,
    flag: u16,
    is_promotion: bool,
    moves: &mut MoveList,
) {
    if is_promotion {
        for to_sq in dest_bb {
            let from_sq = (to_sq as i16 - offset) as u16;
            for pflag in PROMOTION_FLAGS {
                moves.push(Move::new(from_sq, to_sq, flag | pflag));
            }
        }
    } else {
        for to_sq in dest_bb {
            let from_sq = (to_sq as i16 - offset) as u16;
            moves.push(Move::new(from_sq, to_sq, flag));
        }
    }
}
