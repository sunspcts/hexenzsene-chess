use crate::bitboard::Bitboard;
use crate::movegen::magic_sliders::init_magics;
use crate::piece::Piece;

use super::Board;
use super::Side;
use super::state::GameState;

const PIECE_CHARS: &str = "kqrbnpKQRBNP";

// BOARD INIT
impl Board {
    // This function doesn't like being given a malformed fen. I'll add proper error handling before release, I swear.
    // HOWEVER. Since the engine is mostly to be used with GUIs, it doesn't really have many user friendly ways to load a FEN.
    // Any UCI-conformant GUI will be fine.

    pub fn new_from_fen(fen: &str) -> Self {
        init_magics();

        let fen_parts: Vec<&str> = fen.split_ascii_whitespace().collect();
        let (piece_bb, side_bb, mailbox) = init_bb_mb_fen(fen_parts[0]);

        let game_state = GameState {
            active_side: init_active_side(fen_parts[1]),
            castling: init_castling_rights(fen_parts[2]),
            en_passant_square: init_ep_square(fen_parts[3]),
            half_moves: init_halfmoves(fen_parts[4]),
            move_counter: init_move_counter(fen_parts[5]),
            curr_zobrist_key: 0,
        };

        let mut board = Board {
            piece_bb,
            side_bb,
            game_state,
            mailbox,
        };

        board.recompute_zobrist_hash();

        board
    }
}
// FEN PARSING

fn init_bb_mb_fen(fen_part_1: &str) -> ([[Bitboard; 6]; 2], [Bitboard; 2], [Piece; 64]) {
    let mut piece_bb = [[Bitboard::default(); 6]; 2];
    let mut side_bb = [Bitboard::default(); 2];
    let mut mailbox: [Piece; 64] = [Piece::None; 64];

    let mut rank = 7;
    let mut file = 0;
    for char in fen_part_1.chars() {
        let sq = (rank * 8) + file;
        //This could've been a lookup table. But this is more readable, and this is a very rarely used method, so it's fine.
        match char {
            'p' => {
                piece_bb[1][0] |= Bitboard::one() << sq;
                mailbox[sq] = Piece::Pawn
            }
            'P' => {
                piece_bb[0][0] |= Bitboard::one() << sq;
                mailbox[sq] = Piece::Pawn
            }
            'n' => {
                piece_bb[1][1] |= Bitboard::one() << sq;
                mailbox[sq] = Piece::Knight
            }
            'N' => {
                piece_bb[0][1] |= Bitboard::one() << sq;
                mailbox[sq] = Piece::Knight
            }
            'b' => {
                piece_bb[1][2] |= Bitboard::one() << sq;
                mailbox[sq] = Piece::Bishop
            }
            'B' => {
                piece_bb[0][2] |= Bitboard::one() << sq;
                mailbox[sq] = Piece::Bishop
            }
            'r' => {
                piece_bb[1][3] |= Bitboard::one() << sq;
                mailbox[sq] = Piece::Rook
            }
            'R' => {
                piece_bb[0][3] |= Bitboard::one() << sq;
                mailbox[sq] = Piece::Rook
            }
            'q' => {
                piece_bb[1][4] |= Bitboard::one() << sq;
                mailbox[sq] = Piece::Queen
            }
            'Q' => {
                piece_bb[0][4] |= Bitboard::one() << sq;
                mailbox[sq] = Piece::Queen
            }
            'k' => {
                piece_bb[1][5] |= Bitboard::one() << sq;
                mailbox[sq] = Piece::King
            }
            'K' => {
                piece_bb[0][5] |= Bitboard::one() << sq;
                mailbox[sq] = Piece::King
            }
            '1'..='8' => {
                if let Some(x) = char.to_digit(10) {
                    file += x as usize;
                }
            }
            '/' => {
                rank -= 1;
                file = 0
            }
            _ => panic!("unsupported character {} in FEN string!", char), // fix this, please dont just fucking panic
        }

        if PIECE_CHARS.contains(char) {
            file += 1
        }
    }

    //initializing side bitboards
    for side in 0..=1 {
        let piece_bbs = piece_bb[side];
        for bb in piece_bbs {
            side_bb[side] |= bb
        }
    }

    (piece_bb, side_bb, mailbox)
}

fn init_active_side(fen_part_2: &str) -> Side {
    match fen_part_2 {
        "w" => Side::White,
        "b" => Side::Black,
        _ => panic!(
            "unsupported field {} in side_to_play component of FEN string!",
            fen_part_2
        ),
    }
}

fn init_castling_rights(fen_part_3: &str) -> u8 {
    let mut castling_rights = 0;
    for c in fen_part_3.chars() {
        castling_rights += match c {
            'K' => 0b0001,
            'Q' => 0b0010,
            'k' => 0b0100,
            'q' => 0b1000,
            _ => 0,
        }
    }
    castling_rights
}

fn init_ep_square(fen_part_4: &str) -> Option<u8> {
    if fen_part_4 == "-" {
        None
    } else {
        // if the square is invalid, or if it only includes file, we'll get corrupted data. conforms to uci specification though, so it's fine.
        let mut chars = fen_part_4.chars();
        let file_char = chars.next().unwrap();
        let file = file_char as u8 - b'a';
        let rank_char = chars.next().unwrap();
        let rank = rank_char as u8 - b'1';
        Some(rank * 8 + file)
    }
}

fn init_halfmoves(fen_part_5: &str) -> u8 {
    fen_part_5.parse::<u8>().unwrap()
}

fn init_move_counter(fen_part_6: &str) -> u16 {
    fen_part_6.parse::<u16>().unwrap()
}
