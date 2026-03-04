use crate::bitboard::{Bitboard, HEIGHT, WIDTH};

/// Column exploration order: center-first for better alpha-beta pruning.
const COLUMN_ORDER: [usize; WIDTH] = [3, 2, 4, 1, 5, 0, 6];

/// Maximum possible score (win on the first move).
const MAX_SCORE: i32 = (WIDTH * HEIGHT) as i32 / 2;

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

/// Evaluate the board position from the current player's perspective.
fn evaluate(board: &Bitboard) -> i32 {
    let current = board.position_mask();
    let opponent = current ^ board.all_mask();
    let mut score: i32 = 0;

    for col in 0..WIDTH {
        for row in 0..HEIGHT {
            let bit = 1u64 << (col * (HEIGHT + 1) + row);
            if current & bit != 0 {
                score += POSITION_WEIGHTS[col][row];
            } else if opponent & bit != 0 {
                score -= POSITION_WEIGHTS[col][row];
            }
        }
    }

    score
}

/// Negamax with alpha-beta pruning.
///
/// Returns a score from the current player's perspective.
fn negamax(board: &Bitboard, depth: u32, mut alpha: i32, beta: i32) -> i32 {
    // Check if the previous player just won (after play() swapped perspective).
    if board.is_winning() {
        return -(MAX_SCORE - board.move_count() as i32 / 2);
    }

    if board.is_draw() {
        return 0;
    }

    if depth == 0 {
        return evaluate(board);
    }

    // Upper bound pruning: best we can do is win on our next move.
    let max_possible = MAX_SCORE - (board.move_count() as i32 + 1) / 2;
    if max_possible <= alpha {
        return alpha;
    }

    for &col in &COLUMN_ORDER {
        if !board.can_play(col) {
            continue;
        }
        let mut child = *board;
        child.play(col).expect("checked can_play");
        let score = -negamax(&child, depth - 1, -beta, -alpha);
        if score >= beta {
            return score;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

/// Find the best column to play for the current position.
///
/// Searches to the given `depth` using negamax with alpha-beta pruning.
pub fn best_move(board: &Bitboard, depth: u32) -> usize {
    let mut best_col = COLUMN_ORDER[0];
    let mut best_score = i32::MIN + 1;

    for &col in &COLUMN_ORDER {
        if !board.can_play(col) {
            continue;
        }
        let mut child = *board;
        child.play(col).expect("checked can_play");
        let score = -negamax(&child, depth - 1, -(i32::MAX - 1), -best_score);
        if score > best_score {
            best_score = score;
            best_col = col;
        }
    }

    best_col
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wins_immediately_when_possible() {
        // Red has 3 in a row in col 0 (rows 0,1,2), can win by playing col 0 again.
        let mut board = Bitboard::new();
        // R:0, Y:1, R:0, Y:1, R:0, Y:1 => Red has 3 in col 0, Yellow has 3 in col 1.
        for _ in 0..3 {
            board.play(0).unwrap(); // Red
            board.play(1).unwrap(); // Yellow
        }
        // Red to move, should play col 0 for the win.
        assert_eq!(best_move(&board, 3), 0);
    }

    #[test]
    fn blocks_opponent_win() {
        // Yellow has 3 in a row in col 2 (rows 0,1,2). Red must block col 2.
        let mut board = Bitboard::new();
        // R:0, Y:2, R:0, Y:2, R:1, Y:2 => Yellow has 3 in col 2.
        board.play(0).unwrap(); // R
        board.play(2).unwrap(); // Y
        board.play(0).unwrap(); // R
        board.play(2).unwrap(); // Y
        board.play(1).unwrap(); // R
        board.play(2).unwrap(); // Y
        // Red to move. Yellow threatens col 2 row 3. Red must block.
        assert_eq!(best_move(&board, 5), 2);
    }

    #[test]
    fn empty_board_evaluates_to_zero() {
        let board = Bitboard::new();
        assert_eq!(evaluate(&board), 0);
    }

    #[test]
    fn prefers_center_on_empty_board() {
        let board = Bitboard::new();
        let col = best_move(&board, 5);
        // Should prefer center column (3).
        assert_eq!(col, 3);
    }
}
