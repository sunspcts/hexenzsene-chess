use super::*;

#[test]
fn test_make_null_move() {
    crate::movegen::magic_sliders::init_magics();
    let board = Board::new_from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
    let null_board = board.make_null_move();

    assert_eq!(null_board.game_state.active_side, Side::White);
    assert_eq!(null_board.game_state.en_passant_square, None);
    assert_eq!(null_board.game_state.half_moves, 1);
    assert_eq!(null_board.game_state.move_counter, 2);

    let expected_board = Board::new_from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2");
    assert_eq!(null_board.game_state.curr_zobrist_key, expected_board.game_state.curr_zobrist_key);
    assert_eq!(null_board, expected_board);
}