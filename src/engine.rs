use std::{
    io::{self, BufRead},
    sync::{Arc, Mutex, atomic::Ordering},
    thread,
    time::Duration,
};

use crate::{
    board::{Board, Side},
    movegen::magic_sliders::init_magics,
    moves::{Move, MoveList},
    search::{HistoryTable, KillerTable, SearchControl, SearchEnv, TT, search},
};

const ENGINE_NAME: &str = "Hexenzsene v0.2.0";
const ENGINE_AUTHOR: &str = "Averie Harkins";
const DEFAULT_DEPTH: i64 = 8;
const DEFAULT_HASH_MB: usize = 16;
const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// Currently the ONLY public facing function in the crate.
// Handles UCI protocol.

pub fn engine() {
    println!("Initializing...");
    // MAGICS_PTR is null by default. If we don't initialize it, we're gonna have a bad time!
    // We try to initialize this again on board creation.
    init_magics();
    let stdin = io::stdin();
    let mut engine = Engine::new();

    println!("Awaiting UCI Command...");

    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("uci") => engine.print_info(),
            Some("isready") => println!("readyok"),
            Some("setoption") => engine.setoption(line),
            Some("position") => engine.position(line),
            Some("ucinewgame") => engine.ucinewgame(),
            Some("go") => engine.go(line),
            Some("stop") => engine.stop(),
            Some("quit") => {
                engine.stop();
                break;
            }
            Some("perft") => engine.perft(),
            _ => {}
        }
    }
}

struct Engine {
    board: Board,
    hash_history: Vec<u64>,
    search_control: SearchControl,
    search_thread: Option<thread::JoinHandle<()>>,
    tt: Arc<Mutex<TT>>,
    current_age: u8,
}

impl Engine {
    fn new() -> Self {
        let board = Board::new_from_fen(STARTPOS_FEN);
        let hash_history = vec![board.game_state.curr_zobrist_key];
        Self {
            board,
            hash_history,
            search_control: SearchControl::new(),
            search_thread: None,
            tt: Arc::new(Mutex::new(TT::new(DEFAULT_HASH_MB))),
            current_age: 0,
        }
    }

    fn stop_search(&mut self) {
        self.search_control.stop();
        if let Some(handle) = self.search_thread.take() {
            let _ = handle.join();
        }
    }

    fn print_info(&self) {
        println!("id name {}", ENGINE_NAME);
        println!("id author {}", ENGINE_AUTHOR);
        println!(
            "option name Hash type spin default {} min 1 max 1024",
            DEFAULT_HASH_MB
        );
        println!("uciok");
    }

    fn setoption(&mut self, line: &str) {
        self.stop_search();
        if let Some((name, value)) = parse_setoption(line)
            && name.eq_ignore_ascii_case("hash")
            && let Ok(mb) = value.parse::<usize>()
        {
            let mb = mb.clamp(1, 1024);
            *self.tt.lock().unwrap() = TT::new(mb);
        }
    }

    fn position(&mut self, line: &str) {
        self.stop_search();
        (self.board, self.hash_history) = parse_uci_position(self.board, line);
    }

    fn ucinewgame(&mut self) {
        self.stop_search();
        self.board = Board::new_from_fen(STARTPOS_FEN);
        self.hash_history = vec![self.board.game_state.curr_zobrist_key];
        self.current_age = 0;
        self.tt.lock().unwrap().clear();
    }

    fn go(&mut self, line: &str) {
        self.stop_search();
        self.search_control = SearchControl::new();
        self.current_age = self.current_age.wrapping_add(1);

        let params = GoParameters::new(line);
        let max_depth = params.depth.unwrap_or(
            if params.wtime.is_some()
                || params.btime.is_some()
                || params.movetime.is_some()
                || params.infinite
                || params.nodes.is_some()
            {
                200
            } else {
                DEFAULT_DEPTH
            },
        );

        let search_time = calculate_search_time(&self.board, &params);

        if let Some(time_ms) = search_time {
            let stop_clone = self.search_control.stop.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(time_ms));
                stop_clone.store(true, Ordering::Relaxed);
            });
        }

        let new_board = self.board;
        let new_history = self.hash_history.clone();
        let new_control = self.search_control.clone();

        let node_limit = params.nodes.unwrap_or(u64::MAX);
        let tt_clone = self.tt.clone();
        let search_age = self.current_age;

        self.search_thread = Some(thread::spawn(move || {
            let mut tt_guard = tt_clone.lock().unwrap();
            let mut env = SearchEnv {
                nodes_visited: 0,
                node_limit,
                silent: false,
                hash_history: new_history,
                search_control: new_control,
                stopped: false,
                age: search_age,
                move_lists: [MoveList::default(); crate::search::MAX_PLY],
                tt: &mut tt_guard,
                killers: KillerTable::new(),
                history: HistoryTable::new(),
                pv: crate::search::PvTable::new(),
            };

            let (_score, best_move) = search(&new_board, max_depth, &mut env);

            if let Some(mv) = best_move {
                println!("bestmove {}", mv);
            } else {
                let mut moves = MoveList::default();
                moves.generate_pseudolegal_moves(&new_board);
                let fallback = moves.into_iter().find(|m| new_board.make(*m).is_some());
                if let Some(mv) = fallback {
                    println!("bestmove {}", mv);
                } else {
                    println!("bestmove (none)");
                }
            }
        }));
    }

    fn stop(&mut self) {
        self.stop_search();
    }

    fn perft(&self) {
        let board = Board::new_from_fen(STARTPOS_FEN);
        let start = std::time::Instant::now();
        let nodes = board.perft(6);
        let elapsed = start.elapsed();
        println!(
            "perft(6): {} nodes in {:.3?}, {:.2} MNPS",
            nodes,
            elapsed,
            (nodes as f64 / elapsed.as_secs_f64()) / 1_000_000.0
        );
    }
}

#[derive(Default)]
struct GoParameters {
    depth: Option<i64>,
    movetime: Option<u64>,
    nodes: Option<u64>,
    wtime: Option<u64>,
    btime: Option<u64>,
    winc: Option<u64>,
    binc: Option<u64>,
    infinite: bool,
}

impl GoParameters {
    fn new(line: &str) -> GoParameters {
        let mut parts = line.split_whitespace();

        let mut params = Self::default();

        while let Some(part) = parts.next() {
            match part {
                "depth" => {
                    if let Some(val) = parts.next()
                        && let Ok(parsed) = val.parse::<i64>()
                    {
                        params.depth = Some(parsed);
                    }
                }
                "movetime" => {
                    if let Some(val) = parts.next()
                        && let Ok(parsed) = val.parse::<u64>()
                    {
                        params.movetime = Some(parsed);
                    }
                }
                "nodes" => {
                    if let Some(val) = parts.next()
                        && let Ok(parsed) = val.parse::<u64>()
                    {
                        params.nodes = Some(parsed);
                    }
                }
                "wtime" => {
                    if let Some(val) = parts.next()
                        && let Ok(parsed) = val.parse::<u64>()
                    {
                        params.wtime = Some(parsed);
                    }
                }
                "btime" => {
                    if let Some(val) = parts.next()
                        && let Ok(parsed) = val.parse::<u64>()
                    {
                        params.btime = Some(parsed);
                    }
                }
                "winc" => {
                    if let Some(val) = parts.next()
                        && let Ok(parsed) = val.parse::<u64>()
                    {
                        params.winc = Some(parsed);
                    }
                }
                "binc" => {
                    if let Some(val) = parts.next()
                        && let Ok(parsed) = val.parse::<u64>()
                    {
                        params.binc = Some(parsed);
                    }
                }
                "infinite" => params.infinite = true,
                _ => {}
            }
        }

        params
    }
}

fn calculate_search_time(board: &Board, params: &GoParameters) -> Option<u64> {
    if params.infinite {
        return None;
    }
    if let Some(mt) = params.movetime {
        return Some(mt);
    }

    let time_remaining = match board.game_state.active_side {
        Side::White => params.wtime,
        Side::Black => params.btime,
    };

    let inc = match board.game_state.active_side {
        Side::White => params.winc,
        Side::Black => params.binc,
    }
    .unwrap_or(0);

    if let Some(t) = time_remaining {
        match t > 200000 {
            true => return Some(10000),
            false => return Some(t / 20 + inc / 2),
        }
    }

    None
}

fn parse_uci_position(curr_board: Board, line: &str) -> (Board, Vec<u64>) {
    let mut parts = line.split_whitespace();
    let mut board = curr_board;

    let _ = parts.next();
    let mode = parts.next();

    if mode == Some("fen") {
        let fen_parts: Vec<&str> = parts.by_ref().take_while(|part| *part != "moves").collect();
        let fen = fen_parts.join(" ");
        board = Board::new_from_fen(&fen);
    } else if mode == Some("startpos") {
        board = Board::new_from_fen(STARTPOS_FEN);
    }

    let mut hash_history = vec![board.game_state.curr_zobrist_key];

    for m in parts {
        if m == "moves" {
            continue;
        }

        if let Some(mv) = Move::from_uci(&board, m)
            && let Some(next_board) = board.make(mv)
        {
            board = next_board;
            hash_history.push(board.game_state.curr_zobrist_key);
        }
    }
    (board, hash_history)
}

fn parse_setoption(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let name_idx = parts.iter().position(|&p| p.eq_ignore_ascii_case("name"))?;
    let value_idx = parts
        .iter()
        .position(|&p| p.eq_ignore_ascii_case("value"))?;

    if name_idx < value_idx && name_idx + 1 < parts.len() {
        let name = parts[name_idx + 1..value_idx].join(" ");
        let value = parts[value_idx + 1..].join(" ");
        return Some((name, value));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uci_position_from_startpos() {
        let startpos_board = Board::new_from_fen(STARTPOS_FEN);
        // Scotch my beloved <3
        let (board, _) = parse_uci_position(
            startpos_board,
            "position startpos moves e2e4 e7e5 g1f3 b8c6 d2d4",
        );
        let fen_board = Board::new_from_fen(
            "r1bqkbnr/pppp1ppp/2n5/4p3/3PP3/5N2/PPP2PPP/RNBQKB1R b KQkq d3 0 3",
        );

        assert_eq!(board, fen_board)
    }
}
