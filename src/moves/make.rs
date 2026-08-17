use super::*;

use crate::{
    bitboard::Bitboard,
    hashing::{ZOBRIST_RANDOMS, get_piece_zobrist_index},
    movegen::magic_sliders,
};

impl Board {
    pub unsafe fn make(&self, mv: Move) -> Option<Board> {
        let mut board = *self;
        let side = board.game_state.active_side;
        let enemy = side.flip();
        let from = mv.from_sq();
        let to = mv.to_sq();
        let piece = board[from];
        let flags = mv.flags();

        let mut zobrist_delta = 0;

        // are we castling? either direction.
        if flags & 0b1110 == move_flags::KING_CASTLE {
            let transit_sq = match to {
                2 => 3,
                6 => 5,
                58 => 59,
                62 => 61,
                _ => unreachable!(),
            };
            // We check if the to square is attacked at the end of the function anyway.
            // Checking here might give a *tiny* speedup from the early return? Will test at some point but there are more pressing matters ^_^
            if unsafe {
                self.is_attacked(from, enemy)
                    || self.is_attacked(transit_sq, enemy)
                    || self.is_attacked(to, enemy)
            } {
                return None;
            }
        }

        board.game_state.inc_halfmoves();
        // xoring out the old ep square.
        if let Some(old_ep_square) = board.game_state.en_passant_square {
            zobrist_delta ^= ZOBRIST_RANDOMS[768 + 16 + (old_ep_square % 8) as usize];
            board.game_state.en_passant_square = None;
        }

        // capture handling!
        if mv.is_capture() {
            let captured_piece = board[to];
            if captured_piece != Piece::None {
                board.remove_piece(enemy, captured_piece, to);
            }
            board.game_state.reset_halfmoves();
            if captured_piece == Piece::Rook {
                board.update_castling_rights(255, to); //255 is a dummy value here!
            }
        }

        // simple in the case where the piece is not a pawn, just move the fucking thing. if it is a pawn, we need to check for promotions, en passant, and double pushes.
        if piece != Piece::Pawn {
            board.move_piece(piece, side, from, to)
        } else {
            board.remove_piece(side, piece, from);

            // Promotion handling!
            let piece_to_place = if mv.is_promo() {
                promo_flag_parser(flags)
            } else {
                Piece::Pawn
            };

            board.place_piece(side, piece_to_place, to);

            // Pawn moves always reset the clock.
            board.game_state.reset_halfmoves();

            if flags == move_flags::EP_CAPTURE {
                board.remove_piece(enemy, Piece::Pawn, (to as u8 ^ 8) as u16); // I don't know why exactly this xor works, but it does. it's pretty neat!
            }

            if flags == move_flags::DOUBLE_PAWN_PUSH {
                let ep_square = ((from + to) / 2) as u8; // easiest way to calculate intermediate square without any side conditionals.
                board.game_state.en_passant_square = Some(ep_square);
                zobrist_delta ^= ZOBRIST_RANDOMS[768 + 16 + (ep_square % 8) as usize];
            }
        }

        // update castling rights if king or rook moved.
        if piece == Piece::King || piece == Piece::Rook {
            board.update_castling_rights(from, to);
        }

        // castling moves only encode the king move, gotta move the rook as well.
        if flags & 0b1110 == move_flags::KING_CASTLE {
            match to {
                2 => board.move_piece(Piece::Rook, side, 0, 3),
                6 => board.move_piece(Piece::Rook, side, 7, 5),
                58 => board.move_piece(Piece::Rook, side, 56, 59),
                62 => board.move_piece(Piece::Rook, side, 63, 61),
                _ => unreachable!(),
            }
        }

        // increment full move count
        if side == Side::Black {
            board.game_state.inc_count();
        }

        // Apply zobrist delta.
        board.game_state.curr_zobrist_key ^= zobrist_delta;

        // Is the side to move's king attacked? If so, illegal move!
        let king_square =
            (board.piece_bb[side as usize][Piece::King as usize]).trailing_zeros() as u16;
        let is_legal = !unsafe { board.is_attacked(king_square, enemy) };

        if is_legal {
            board.game_state.active_side = enemy;
            // Apply side to move zobrist.
            board.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[768 + 16 + 8];
            Some(board)
        } else {
            None
        }
    }

    // helpers.
    fn remove_piece(&mut self, side: Side, piece: Piece, sq: u16) {
        let mask = Bitboard::one() << sq as usize;
        let side_idx = side as usize;
        let piece_idx = piece as usize;

        self.piece_bb[side_idx][piece_idx] ^= mask;
        self.side_bb[side_idx] ^= mask;
        self[sq] = Piece::None;

        //gotta update the hash.
        let zobrist_idx = get_piece_zobrist_index(piece, side, sq as usize);
        self.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[zobrist_idx];
    }

    fn place_piece(&mut self, side: Side, piece: Piece, sq: u16) {
        let mask = Bitboard::one() << sq as usize;
        let side_idx = side as usize;
        let piece_idx = piece as usize;
        self.piece_bb[side_idx][piece_idx] |= mask;
        self.side_bb[side_idx] |= mask;
        self[sq] = piece;

        let zobrist_idx = get_piece_zobrist_index(piece, side, sq as usize);
        self.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[zobrist_idx];
    }

    fn move_piece(&mut self, piece: Piece, side: Side, from: u16, to: u16) {
        self.remove_piece(side, piece, from);
        self.place_piece(side, piece, to);
    }

    pub fn perft(&self, depth: u8) -> u64 {
        magic_sliders::init_magics(); // safety :D
        let mut move_lists = [MoveList::default(); 256];
        unsafe { self.perft_helper(depth, 0, &mut move_lists) }
    }

    unsafe fn perft_helper(&self, depth: u8, ply: usize, move_lists: &mut [MoveList; 256]) -> u64 {
        if depth == 0 {
            return 1;
        }

        let ply_idx = ply.min(255);
        move_lists[ply_idx].generate_pseudolegal_moves(self);
        let moves = move_lists[ply_idx];

        if depth == 1 {
            let side = self.game_state.active_side;
            let enemy = side.flip();
            let king_sq =
                (self.piece_bb[side as usize][Piece::King as usize]).trailing_zeros() as u16;

            if king_sq < 64 && !unsafe { self.is_attacked(king_sq, enemy) } {
                let pinned_bb = unsafe { self.pinned_bitboard(side) };

                let mut count = 0;
                for &m in &moves {
                    let from = m.from_sq();
                    let is_pinned = (pinned_bb & Bitboard::new(1u64 << from)) != Bitboard::zero();
                    let is_king_or_ep = from == king_sq || m.flags() == move_flags::EP_CAPTURE;

                    if is_king_or_ep || is_pinned {
                        if unsafe { self.make(m).is_some() } {
                            count += 1;
                        }
                    } else {
                        count += 1;
                    }
                }
                return count;
            }

            let mut count = 0;
            for &m in &moves {
                if unsafe { self.make(m).is_some() } {
                    count += 1;
                }
            }
            return count;
        }

        let mut nodes = 0;
        for &m in &moves {
            if let Some(next_board) = unsafe { self.make(m) } {
                nodes += unsafe { next_board.perft_helper(depth - 1, ply + 1, move_lists) };
            }
        }

        nodes
    }
}

// another nice helper!
fn promo_flag_parser(flag: u16) -> Piece {
    match flag & 0b0011 {
        0 => Piece::Knight,
        1 => Piece::Bishop,
        2 => Piece::Rook,
        3 => Piece::Queen,
        _ => unreachable!(),
    }
}
