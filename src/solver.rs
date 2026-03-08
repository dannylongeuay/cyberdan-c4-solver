use crate::bitboard::{Bitboard, HEIGHT, WIDTH};
use log::debug;
use std::time::{Duration, Instant};

/// Column exploration order: center-first for better alpha-beta pruning.
const COLUMN_ORDER: [usize; WIDTH] = [3, 2, 4, 1, 5, 0, 6];

/// Win score offset, kept well above the heuristic evaluation range (~±276)
/// so that win/loss scores never overlap with positional scores.
const WIN_SCORE: i32 = 1_000;

/// How often to check the clock (amortizes Instant::now() cost).
const CHECK_INTERVAL: u64 = 1024;

/// Positional weight table: approximates how many 4-in-a-row lines pass through each cell.
/// Indexed as [col][row] where row 0 is the bottom.
const POSITION_WEIGHTS: [[i32; HEIGHT]; WIDTH] = [
    [3, 4, 5, 5, 4, 3],
    [4, 6, 8, 8, 6, 4],
    [5, 8, 11, 11, 8, 5],
    [7, 10, 13, 13, 10, 7],
    [5, 8, 11, 11, 8, 5],
    [4, 6, 8, 8, 6, 4],
    [3, 4, 5, 5, 4, 3],
];

/// Number of entries in the transposition table (~16 MB).
const TT_SIZE: usize = 1 << 20;

/// Bound type stored in a transposition table entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

/// A single transposition table entry.
#[derive(Debug, Clone, Copy)]
struct TTEntry {
    key: u64,
    score: i32,
    depth: u8,
    flag: TTFlag,
    best_move: u8,
}

impl Default for TTEntry {
    fn default() -> Self {
        TTEntry {
            key: 0,
            score: 0,
            depth: 0,
            flag: TTFlag::Exact,
            best_move: 0,
        }
    }
}

/// Fixed-size transposition table with always-replace policy.
struct TranspositionTable {
    entries: Vec<TTEntry>,
    mask: usize,
}

impl TranspositionTable {
    fn new(size: usize) -> Self {
        debug_assert!(size.is_power_of_two());
        TranspositionTable {
            entries: vec![TTEntry::default(); size],
            mask: size - 1,
        }
    }

    fn probe(&self, key: u64) -> Option<&TTEntry> {
        let entry = &self.entries[key as usize & self.mask];
        if entry.key == key {
            Some(entry)
        } else {
            None
        }
    }

    fn store(&mut self, entry: TTEntry) {
        self.entries[entry.key as usize & self.mask] = entry;
    }
}

/// Mutable state threaded through the search tree.
struct SearchState {
    deadline: Instant,
    node_count: u64,
    timed_out: bool,
    tt: TranspositionTable,
}

/// Result of a completed root-level search at a given depth.
struct SearchResult {
    best_col: usize,
    best_score: i32,
}

/// Evaluate the board position from the current player's perspective.
fn evaluate(board: &Bitboard) -> i32 {
    let current = board.position_mask();
    let opponent = current ^ board.all_mask();
    let mut score: i32 = 0;

    for (col, col_weights) in POSITION_WEIGHTS.iter().enumerate() {
        for (row, &weight) in col_weights.iter().enumerate() {
            let bit = 1u64 << (col * (HEIGHT + 1) + row);
            if current & bit != 0 {
                score += weight;
            } else if opponent & bit != 0 {
                score -= weight;
            }
        }
    }

    score
}

/// Negamax with alpha-beta pruning, transposition table, and timeout support.
///
/// Uses fail-soft: returns `best_score` which may lie outside [alpha, beta].
/// If `state.timed_out` is set, returns a dummy value (0) and the caller
/// must discard the entire depth's result.
fn negamax(
    board: &Bitboard,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    state: &mut SearchState,
) -> i32 {
    state.node_count += 1;
    if state.node_count.is_multiple_of(CHECK_INTERVAL) && Instant::now() >= state.deadline {
        state.timed_out = true;
    }
    if state.timed_out {
        return 0;
    }

    // Check if the previous player just won (after play() swapped perspective).
    if board.is_winning() {
        return -(WIN_SCORE - board.move_count() as i32 / 2);
    }

    if board.is_draw() {
        return 0;
    }

    if depth == 0 {
        return evaluate(board);
    }

    // Upper bound pruning: best we can do is win on our next move.
    let max_possible = WIN_SCORE - (board.move_count() as i32 + 1) / 2;
    if max_possible <= alpha {
        return alpha;
    }

    let key = board.key();
    let original_alpha = alpha;
    let mut tt_best_move: Option<usize> = None;

    // Probe TT
    if let Some(entry) = state.tt.probe(key) {
        tt_best_move = Some(entry.best_move as usize);
        if entry.depth as u32 >= depth {
            match entry.flag {
                TTFlag::Exact => return entry.score,
                TTFlag::LowerBound => {
                    if entry.score >= beta {
                        return entry.score;
                    }
                    if entry.score > alpha {
                        alpha = entry.score;
                    }
                }
                TTFlag::UpperBound => {
                    if entry.score <= alpha {
                        return entry.score;
                    }
                }
            }
        }
    }

    let mut best_score = i32::MIN + 1;
    let mut best_col: u8 = COLUMN_ORDER[0] as u8;

    // Build move order: TT best move first, then remaining columns.
    let mut move_order = [0usize; WIDTH];
    let mut move_count = 0;

    if let Some(tt_col) = tt_best_move {
        if tt_col < WIDTH && board.can_play(tt_col) {
            move_order[move_count] = tt_col;
            move_count += 1;
        }
    }
    for &col in &COLUMN_ORDER {
        if Some(col) == tt_best_move {
            continue;
        }
        if !board.can_play(col) {
            continue;
        }
        move_order[move_count] = col;
        move_count += 1;
    }

    for &col in &move_order[..move_count] {
        let mut child = *board;
        child.play(col).expect("checked can_play");
        let score = -negamax(&child, depth - 1, -beta, -alpha, state);
        if state.timed_out {
            return 0;
        }
        if score > best_score {
            best_score = score;
            best_col = col as u8;
        }
        if score >= beta {
            break;
        }
        if score > alpha {
            alpha = score;
        }
    }

    // Store in TT (skip if timed out).
    if !state.timed_out {
        let flag = if best_score <= original_alpha {
            TTFlag::UpperBound
        } else if best_score >= beta {
            TTFlag::LowerBound
        } else {
            TTFlag::Exact
        };
        state.tt.store(TTEntry {
            key,
            score: best_score,
            depth: depth as u8,
            flag,
            best_move: best_col,
        });
    }

    best_score
}

/// Search all root moves at a fixed depth. Returns `None` if the search timed out.
fn search_at_depth(board: &Bitboard, depth: u32, state: &mut SearchState) -> Option<SearchResult> {
    let mut best_col = COLUMN_ORDER[0];
    let mut best_score = i32::MIN + 1;

    // Build root move order: TT best move first, then remaining columns.
    let key = board.key();
    let tt_best_move = state.tt.probe(key).map(|e| e.best_move as usize);

    let mut move_order = [0usize; WIDTH];
    let mut move_count = 0;

    if let Some(tt_col) = tt_best_move {
        if tt_col < WIDTH && board.can_play(tt_col) {
            move_order[move_count] = tt_col;
            move_count += 1;
        }
    }
    for &col in &COLUMN_ORDER {
        if Some(col) == tt_best_move {
            continue;
        }
        if !board.can_play(col) {
            continue;
        }
        move_order[move_count] = col;
        move_count += 1;
    }

    let mut alpha = i32::MIN + 1;

    for &col in &move_order[..move_count] {
        let mut child = *board;
        child.play(col).expect("checked can_play");
        let score = -negamax(&child, depth - 1, -(i32::MAX - 1), -alpha, state);
        if state.timed_out {
            return None;
        }
        if score > best_score {
            best_score = score;
            best_col = col;
        }
        if score > alpha {
            alpha = score;
        }
    }

    // Store root result as Exact (full window search).
    state.tt.store(TTEntry {
        key,
        score: best_score,
        depth: depth as u8,
        flag: TTFlag::Exact,
        best_move: best_col as u8,
    });

    Some(SearchResult {
        best_col,
        best_score,
    })
}

/// Find the best column to play for the current position.
///
/// Uses iterative deepening from depth 1 up to `max_depth`, stopping early
/// if the timeout is reached. Returns the best move from the deepest fully
/// completed search.
pub fn best_move(board: &Bitboard, max_depth: u32, timeout: Duration) -> usize {
    let mut state = SearchState {
        deadline: Instant::now() + timeout,
        node_count: 0,
        timed_out: false,
        tt: TranspositionTable::new(TT_SIZE),
    };

    // Fallback: first playable column in center-first order.
    let fallback = COLUMN_ORDER
        .iter()
        .copied()
        .find(|&c| board.can_play(c))
        .unwrap_or(3);
    let mut best = SearchResult {
        best_col: fallback,
        best_score: i32::MIN + 1,
    };

    for depth in 1..=max_depth {
        match search_at_depth(board, depth, &mut state) {
            Some(result) => {
                debug!(
                    "  depth {:2}: best_col={} score={:6} nodes={}",
                    depth,
                    result.best_col + 1,
                    result.best_score,
                    state.node_count
                );
                best = result;
                // Early exit if we found a forced win.
                if best.best_score >= WIN_SCORE - (board.move_count() as i32 + 1) / 2 {
                    debug!("  found forced win");
                    break;
                }
            }
            None => {
                debug!(
                    "  depth {:2}: timed out, using depth {} result",
                    depth,
                    depth - 1
                );
                break;
            }
        }
    }

    best.best_col
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::Player;

    #[test]
    fn wins_immediately_when_possible() {
        let board = Bitboard::from_ascii(
            "
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             R  Y  .  .  .  .  .
             R  Y  .  .  .  .  .
             R  Y  .  .  .  .  .
            ",
            Player::Red,
        );
        assert_eq!(best_move(&board, 3, Duration::from_secs(30)), 0);
    }

    #[test]
    fn blocks_opponent_win() {
        let board = Bitboard::from_ascii(
            "
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  Y  .  .  .  .
             R  .  Y  .  .  .  .
             R  R  Y  .  .  .  .
            ",
            Player::Red,
        );
        // Red to move. Yellow threatens col 2 row 3. Red must block.
        assert_eq!(best_move(&board, 5, Duration::from_secs(30)), 2);
    }

    #[test]
    fn empty_board_evaluates_to_zero() {
        let board = Bitboard::new();
        assert_eq!(evaluate(&board), 0);
    }

    #[test]
    fn prefers_center_on_empty_board() {
        let board = Bitboard::new();
        let col = best_move(&board, 5, Duration::from_secs(30));
        // Should prefer center column (3).
        assert_eq!(col, 3);
    }
}
