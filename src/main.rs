use hexenzsene_chess::engine;
use hexenzsene_chess::movegen::magic_sliders::init_magics;
fn main() {
    println!("Initializing...");
    init_magics();
    println!("Ferrite Chess v0.2.0");
    engine::engine();
}
