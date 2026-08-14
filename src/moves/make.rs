use super::*;

use crate::bitboard::Bitboard;

impl Board {
    pub fn make(&self, mv: Move) -> Option<Board> {
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
            if self.is_attacked(from, enemy) || self.is_attacked(transit_sq, enemy) || self.is_attacked(to, enemy){
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

        // Just doing this on every king/rook move for now. Might change the mechanism but for now it's cool.
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
                _ => unreachable!()
            }
        }

        // increment full move count
        if side == Side::Black {
            board.game_state.inc_count();
        }

        // Switch the active side and update the hash.
        board.game_state.active_side = enemy;
        board.game_state.curr_zobrist_key ^= zobrist_delta ^ ZOBRIST_RANDOMS[768 + 16 + 8];

        let king_square = board.piece_bb[side as usize][Piece::King as usize].trailing_zeros() as u16;
        let is_legal = !board.is_attacked(king_square, enemy);

        // There should be a way to filter obviously illegal moves that runs before this.
        if !is_legal {
            return None
        }

        Some(board)
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
        let mut move_lists = [MoveList::default(); 256];
        self.perft_helper(depth, 0, &mut move_lists)
    }

    fn perft_helper(&self, depth: u8, ply: usize, move_lists: &mut [MoveList; 256]) -> u64 {
        if depth == 0 {
            return 1;
        }

        let ply_idx = ply.min(255);
        self.generate_pseudolegal_moves(&mut move_lists[ply_idx]);
        let moves = move_lists[ply_idx];

        if depth == 1 {
            let mut count = 0;
            for &m in &moves {
                if self.make(m).is_some() {
                    count += 1;
                }
            }
            return count;
        }

        let mut nodes = 0;
        for &m in &moves {
            if let Some(next_board) = self.make(m) {
                nodes += next_board.perft_helper(depth - 1, ply + 1, move_lists);
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
        _ => unreachable!()
    }
}