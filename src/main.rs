mod bitboard;
mod board;
mod engine;
mod eval;
mod heuristics;
mod movegen;
mod moves;
mod piece;
mod search;

fn main() {
    engine::engine();
}
