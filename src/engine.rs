use std::{io::{self, BufRead}, sync::{Arc, Mutex, atomic::Ordering}, thread, time::Duration};

use crate::{board::{Board, Side}, moves::{Move, MoveList}, search::{SearchControl, SearchEnv, TT, search}};

const ENGINE_NAME: &str = "Hexenzsene v0.1.1";
const ENGINE_AUTHOR: &str = "Averie Harkins";
const DEFAULT_DEPTH: i64 = 8;
const DEFAULT_HASH_MB: usize = 16;
const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

//Handles UCI.
pub fn engine() {
    let stdin = io::stdin();
    let mut board = Board::new_from_fen(STARTPOS_FEN);
    let mut hash_history = vec![board.game_state.curr_zobrist_key];
    let mut search_control = SearchControl::new();
    let mut search_thread: Option<thread::JoinHandle<()>> = None;
    let tt = Arc::new(Mutex::new(TT::new(DEFAULT_HASH_MB)));
    let mut current_age: u8 = 0;

    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("uci") => {
                println!("id name {}", ENGINE_NAME);
                println!("id author {}", ENGINE_AUTHOR);
                println!("option name Hash type spin default {} min 1 max 1024", DEFAULT_HASH_MB);
                println!("uciok");
            }
            Some("isready") => println!("readyok"),
            Some("setoption") => {
                stop_search(&mut search_thread, &mut search_control);
                if let Some((name, value)) = parse_setoption(line) {
                    if name.eq_ignore_ascii_case("hash") {
                        if let Ok(mb) = value.parse::<usize>() {
                            let mb = mb.clamp(1, 1024);
                            *tt.lock().unwrap() = TT::new(mb);
                        }
                    }
                }
            }
            Some("position") => {
                stop_search(&mut search_thread, &mut search_control);
                (board, hash_history) = parse_uci_position(board, line);
            }
            Some("ucinewgame") => {
                stop_search(&mut search_thread, &mut search_control);
                board = Board::new_from_fen(STARTPOS_FEN);
                hash_history = vec![board.game_state.curr_zobrist_key];
                current_age = 0;
                tt.lock().unwrap().clear();
            }
            Some("go") => {
                stop_search(&mut search_thread, &mut search_control);
                search_control = SearchControl::new();
                current_age = current_age.wrapping_add(1);
                let params = GoParameters::new(line);
                let max_depth = params.depth.unwrap_or(
                    if params.wtime.is_some() || params.btime.is_some() || params.movetime.is_some() || params.infinite || params.nodes.is_some() {
                        200
                    } else {
                        DEFAULT_DEPTH
                    }
                );

                let search_time = calculate_search_time(&board, &params);

                if let Some(time_ms) = search_time {
                    let stop_clone = search_control.stop.clone();
                    thread::spawn(move || {
                        thread::sleep(Duration::from_millis(time_ms));
                        stop_clone.store(true, Ordering::Relaxed);
                    });
                }

                let new_board = board;
                let new_history = hash_history.clone();
                let new_control = search_control.clone();

                let node_limit = params.nodes.unwrap_or(u64::MAX);
                let tt_clone = tt.clone();
                let search_age = current_age;

                search_thread = Some(thread::spawn(move || {
                    let mut tt_guard = tt_clone.lock().unwrap();
                    let mut env = SearchEnv {
                        nodes_visited: 0,
                        node_limit,
                        hash_history: new_history,
                        search_control: new_control,
                        stopped: false,
                        age: search_age,
                        move_lists: [MoveList::default(); crate::search::MAX_PLY],
                        tt: &mut *tt_guard,
                        killers: [[0; 2]; crate::search::MAX_PLY],
                        history: [[[0; 64]; 64]; 2],
                        pv_table: [[Move::new_from_raw(0); crate::search::MAX_PLY]; crate::search::MAX_PLY],
                        pv_length: [0; crate::search::MAX_PLY],
                    };

                    let (_score, best_move) = search(&new_board, max_depth, &mut env);

                    if let Some(mv) = best_move {
                        println!("bestmove {}", mv);
                    } else {
                        let fallback = new_board
                            .generate_pseudolegal_moves_list()
                            .into_iter()
                            .find(|m| new_board.make(*m).is_some());

                        if let Some(mv) = fallback {
                            println!("bestmove {}", mv);
                        } else {
                            println!("bestmove (none)");
                        }
                    }
                }));
            }
            Some("stop") => {
                stop_search(&mut search_thread, &mut search_control);
                if let Some(handle) = search_thread.take() {
                    let _ = handle.join();
                }
            }
            Some("quit") => {
                stop_search(&mut search_thread, &mut search_control);
                if let Some(handle) = search_thread.take() {
                    let _ = handle.join();
                }
                break;
            }
            Some("perft") => {
                let board = Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
                let start = std::time::Instant::now();
                let nodes = board.perft(6);
                let elapsed = start.elapsed();
                println!("perft(6): {} nodes in {:.3?}, {:.2} MNPS", nodes, elapsed, (nodes as f64 / elapsed.as_secs_f64()) / 1_000_000.0);
            }
            _ => {}
        }
    }

    let _ = (board, hash_history);
}

pub fn stop_search(search_thread: &mut Option<thread::JoinHandle<()>>, search_control: &mut SearchControl) {
    search_control.stop();
    if let Some(handle) = search_thread.take() {
        let _ = handle.join();
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
    infinite: bool
}

impl GoParameters {
    fn new(line: &str) -> GoParameters {
        let mut parts = line.split_whitespace();

        let mut params = Self::default();

        while let Some(part) = parts.next() {
            match part {
                "depth" => {
                    if let Some(val) = parts.next() {
                        if let Ok(parsed) = val.parse::<i64>() {
                            params.depth = Some(parsed);
                        }
                    }
                }
                "movetime" => {
                    if let Some(val) = parts.next() {
                        if let Ok(parsed) = val.parse::<u64>() {
                            params.movetime = Some(parsed);
                        }
                    }
                }
                "nodes" => {
                    if let Some(val) = parts.next() {
                        if let Ok(parsed) = val.parse::<u64>() {
                            params.nodes = Some(parsed);
                        }
                    }
                }
                "wtime" => {
                    if let Some(val) = parts.next() {
                        if let Ok(parsed) = val.parse::<u64>() {
                            params.wtime = Some(parsed);
                        }
                    }
                }
                "btime" => {
                    if let Some(val) = parts.next() {
                        if let Ok(parsed) = val.parse::<u64>() {
                            params.btime = Some(parsed);
                        }
                    }
                }
                "winc" => {
                    if let Some(val) = parts.next() {
                        if let Ok(parsed) = val.parse::<u64>() {
                            params.winc = Some(parsed);
                        }
                    }
                }
                "binc" => {
                    if let Some(val) = parts.next() {
                        if let Ok(parsed) = val.parse::<u64>() {
                            params.binc = Some(parsed);
                        }
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
    }.unwrap_or(0);

    if let Some(t) = time_remaining{
        match t > 200000 {
            true => return Some(10000),
            false => return Some(t / 20 + inc / 2)
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
            continue
        }

        if let Some(mv) = Move::from_uci(&board, m) {
            if let Some(next_board) = board.make(mv) {
                board = next_board;
                hash_history.push(board.game_state.curr_zobrist_key);
            }
        }
    }
    (board, hash_history)
}

fn parse_setoption(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let name_idx = parts.iter().position(|&p| p.eq_ignore_ascii_case("name"))?;
    let value_idx = parts.iter().position(|&p| p.eq_ignore_ascii_case("value"))?;

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
        let (board, _) = parse_uci_position(startpos_board, "position startpos moves e2e4 e7e5 g1f3 b8c6 d2d4");
        let fen_board = Board::new_from_fen("r1bqkbnr/pppp1ppp/2n5/4p3/3PP3/5N2/PPP2PPP/RNBQKB1R b KQkq d3 0 3");

        assert_eq!(board, fen_board)
    }
}