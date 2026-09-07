use crate::{board::Board, eval::eval, moves::Move};

use super::lmr::LM_REDUCTIONS_TABLE;
use super::tt::{NodeType, TTEntry, score_to_tt};
use super::{MATE_EVAL, MAX_PLY, SearchContext, Searcher};

impl<'a> Searcher<'a> {
    pub(super) fn negamax(&mut self, board: &Board, mut context: SearchContext) -> i64 {
        let ply = context.ply().min(MAX_PLY - 1);
        self.pv.clear_ply(ply);

        if self.step_node_and_check() || self.is_draw(board, ply) {
            return 0;
        }

        // Check extensions & depth check
        let in_check = board.is_in_check();
        let depth = context.depth + in_check as i64;
        context.lmr_allowed = depth >= 3 && !in_check;

        // We've reached the end of the search depth, so we switch to quiescence search.
        if depth <= 0 {
            return self.quiescence(board, context);
        }

        // Is this node in the TT?
        let (tt_move, tt_cutoff) =
            self.probe_tt_cutoff(board.game_state.curr_zobrist_key, &context, depth, ply);
        if let Some(score) = tt_cutoff {
            return score;
        }

        // Reverse Futility and Null Move Pruning
        if let Some(score) = self.rfp_nmp(board, &context, depth, ply, in_check) {
            return score;
        }

        // Move Generation & Ordering
        self.generate_and_order_moves(board, tt_move, ply, context.is_pv);

        let old_alpha = context.alpha; // Grab this before we mutate context.alpha in iterate_moves. Needed for TT entry node type determination.
        let movedata = self.iterate_moves(board, &mut context, depth, ply);

        if self.stopped {
            return 0;
        }

        if movedata.move_count == 0 {
            return if in_check { -MATE_EVAL + ply as i64 } else { 0 };
        }

        self.store_tt_entry(board, &movedata, &context, old_alpha, depth);

        movedata.max_score
    }

    #[inline]
    fn probe_tt_cutoff(
        &self,
        zobrist_key: u64,
        context: &SearchContext,
        depth: i64,
        ply: usize,
    ) -> (Option<Move>, Option<i64>) {
        let tt_entry = self.tt.get(zobrist_key);
        let tt_move = tt_entry.and_then(|e| e.best_move());
        let cutoff_score = if ply > 0 {
            tt_entry.and_then(|e| e.cutoff(context.alpha, context.beta, depth, ply as i64))
        } else {
            None
        };
        (tt_move, cutoff_score)
    }

    #[inline]
    fn rfp_nmp(
        &mut self,
        board: &Board,
        context: &SearchContext,
        depth: i64,
        ply: usize,
        in_check: bool,
    ) -> Option<i64> {
        if in_check || !board.has_non_pawn_material(board.game_state.active_side) {
            return None;
        }

        let static_eval = eval(board);
        let margin = 100 * depth;

        if !context.is_pv
            && depth <= 6
            && context.beta < MATE_EVAL - 100
            && static_eval >= context.beta + margin
        {
            self.store_tt_cutoff(board.game_state.curr_zobrist_key, static_eval, depth, ply);
            return Some(static_eval);
        }

        let can_nmp =
            context.nmp_allowed && !context.is_pv && depth >= 3 && context.beta < MATE_EVAL - 100;

        if !can_nmp || static_eval < context.beta {
            return None;
        }

        let null_board = board.make_null()?;
        let reduction = 2 + depth / 6;
        let null_depth = (depth - 1 - reduction).max(0);

        let null_context = context.next_context_null_move(null_depth);

        self.hash_history.push(board.game_state.curr_zobrist_key);
        let null_score = -self.negamax(&null_board, null_context);
        self.hash_history.pop();

        if self.stopped {
            return Some(0);
        }

        if null_score >= context.beta {
            self.store_tt_cutoff(board.game_state.curr_zobrist_key, context.beta, depth, ply);
            return Some(context.beta);
        }

        None
    }

    #[inline(always)]
    fn search_single_move(
        &mut self,
        context: &SearchContext,
        board: &Board,
        depth: i64,
        move_count: usize,
        is_quiet: bool,
        is_killer: bool,
    ) -> i64 {
        // 1. First move: search with full PV window
        if move_count == 0 {
            return -self.negamax(board, context.next_context(depth, context.is_pv));
        }

        // 2. Determine LMR reduction
        let can_reduce = context.lmr_allowed && is_quiet && !is_killer && move_count >= 3;
        let reduction = if can_reduce {
            let d = (context.depth as usize).min(63);
            let m = move_count.min(63);
            LM_REDUCTIONS_TABLE[d][m]
        } else {
            0
        };

        // 3. Search with null window.
        let mut score = -self.negamax(board, context.next_context_null_window(depth - reduction));

        // 4. LMR re-search: if reduced search failed high, re-search at full depth with null window
        if reduction > 0 && score > context.alpha {
            score = -self.negamax(board, context.next_context_null_window(depth));
        }

        // 5. PVS re-search: if null window search beat alpha in a PV node, re-search with full window
        if context.is_pv && score > context.alpha && score < context.beta {
            score = -self.negamax(board, context.next_context(depth, true));
        }

        score
    }

    pub(super) fn iterate_moves(
        &mut self,
        board: &Board,
        context: &mut SearchContext,
        depth: i64,
        ply: usize,
    ) -> MoveData {
        let mut movedata = MoveData::new();

        for i in 0..self.move_lists[ply].len() {
            let candidate_move = self.move_lists[ply].pick_best(i);
            let Some(next_board) = board.make(candidate_move) else {
                continue;
            };

            let is_quiet = !candidate_move.is_capture();
            let is_killer = is_quiet && self.killers.is_killer(ply, candidate_move);

            if is_quiet {
                movedata.record_quiet(candidate_move);
            }

            self.hash_history.push(board.game_state.curr_zobrist_key);
            let score = self.search_single_move(
                context,
                &next_board,
                depth - 1,
                movedata.move_count,
                is_quiet,
                is_killer,
            );
            self.hash_history.pop();

            movedata.move_count += 1;

            if self.stopped {
                break;
            }

            if score > movedata.max_score {
                movedata.max_score = score;
                movedata.best_move = Some(candidate_move);
            }

            if score > context.alpha {
                context.alpha = score;
                self.pv.update(ply, candidate_move);
            }

            if score >= context.beta {
                if is_quiet {
                    self.killers.add(ply, candidate_move);
                    let side = board.game_state.active_side as usize;
                    self.history.update_cutoff(side, candidate_move, movedata.quiets_to_penalize(), depth);
                }
                break;
            }
        }

        movedata
    }

    #[inline]
    fn store_tt_cutoff(&mut self, zobrist_key: u64, score: i64, depth: i64, ply: usize) {
        if !self.stopped {
            self.tt.store(TTEntry {
                zobrist_key,
                score: score_to_tt(score, ply as i64),
                move_data: 0,
                depth: depth as i8,
                node_type: NodeType::LowerBound,
                age: self.age,
            });
        }
    }

    #[inline]
    pub(super) fn store_tt_entry(
        &mut self,
        board: &Board,
        movedata: &MoveData,
        context: &SearchContext,
        old_alpha: i64,
        depth: i64,
    ) {
        if !self.stopped {
            let zobrist_key = board.game_state.curr_zobrist_key;
            let score = movedata.max_score;
            let ply = context.ply() as i64;
            let node_type = context.node_type(score, old_alpha);

            self.tt.store(TTEntry {
                zobrist_key,
                score: score_to_tt(score, ply),
                move_data: movedata.best_move.map(|m| m.data()).unwrap_or(0),
                depth: depth as i8,
                node_type,
                age: self.age,
            });
        }
    }
}

// Convenience / Readability struct.
#[derive(Debug)]
pub(super) struct MoveData {
    pub best_move: Option<Move>,
    pub max_score: i64,
    pub move_count: usize,
    quiet_moves_tried: [Move; 64],
    quiet_count: usize,
}

impl MoveData {
    pub fn new() -> Self {
        Self {
            best_move: None,
            max_score: i64::MIN,
            move_count: 0,
            quiet_moves_tried: [Move::new_from_raw(0); 64],
            quiet_count: 0,
        }
    }

    #[inline(always)]
    pub fn record_quiet(&mut self, candidate_move: Move) {
        if self.quiet_count < 64 {
            self.quiet_moves_tried[self.quiet_count] = candidate_move;
            self.quiet_count += 1;
        }
    }

    #[inline(always)]
    pub fn quiets_to_penalize(&self) -> &[Move] {
        if self.quiet_count > 0 {
            &self.quiet_moves_tried[..self.quiet_count - 1]
        } else {
            &[]
        }
    }
}
