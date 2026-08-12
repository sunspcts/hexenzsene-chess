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
    println!("Ferrite Chess v0.2.0");
    engine::engine();
}
