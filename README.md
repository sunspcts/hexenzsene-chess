# Hexenzsene

A Fully UCI compatible Rust based HCE chess engine written from scratch.

## Features (outdated)

### Board Representation

- Bitboard board representation

### Search

- Negamax with iterative deepening
- Alpha-Beta pruning
- Quiescence Search (optimized captures-only movegen)
- Check extensions
- Transposition table cutoffs

### Heuristics & Move Ordering

- Transposition table move ordering
- MVV-LVA (Most Valuable Victim - Least Valuable Aggressor) heuristic
- Killer move heuristic
- History heuristic with gravity

### Evaluation

- PeSTO Middlegame Piece Square Tables
- Material Evaluation
