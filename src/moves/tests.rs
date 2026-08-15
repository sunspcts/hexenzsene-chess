use crate::board::Board;

#[test]
fn perft_startpos() {
    crate::movegen::magic_sliders::init_magics();
    let board = Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let start = std::time::Instant::now();
    let nodes = board.perft(5);
    let elapsed = start.elapsed();
    println!("perft(5): {} nodes in {:.3?}, {:.2} MNPS", nodes, elapsed, (nodes as f64 / elapsed.as_secs_f64()) / 1_000_000.0);
    assert_eq!(nodes, 4865609);
}