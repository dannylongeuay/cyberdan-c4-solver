/// Number of columns on the board.
pub const WIDTH: usize = 7;
/// Number of rows on the board.
pub const HEIGHT: usize = 6;

/// Players in a Connect Four game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    Red,
    Yellow,
}

impl Player {
    pub fn other(self) -> Player {
        match self {
            Player::Red => Player::Yellow,
            Player::Yellow => Player::Red,
        }
    }
}

/// Errors that can occur when attempting a move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveError {
    ColumnFull,
    OutOfRange,
}

/// Bitboard representation of a Connect Four board.
///
/// Uses column-major layout with 7 bits per column (6 playable rows + 1 sentinel).
/// - `position`: bitmask of the current player's pieces
/// - `mask`: bitmask of all pieces on the board
/// - Opponent's pieces = `position ^ mask`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bitboard {
    position: u64,
    mask: u64,
    current: Player,
    moves: u32,
}

impl Bitboard {
    /// Creates a new empty board with Red to move first.
    pub fn new() -> Self {
        Bitboard {
            position: 0,
            mask: 0,
            current: Player::Red,
            moves: 0,
        }
    }

    /// Returns the player whose turn it is.
    pub fn current_player(&self) -> Player {
        self.current
    }

    /// Returns the number of moves played so far.
    pub fn move_count(&self) -> u32 {
        self.moves
    }

    /// Returns whether a move can be played in the given column.
    pub fn can_play(&self, col: usize) -> bool {
        if col >= WIDTH {
            return false;
        }
        // Check if the top cell of the column is empty.
        (self.mask & top_mask(col)) == 0
    }

    /// Returns a list of columns that are not full.
    pub fn valid_columns(&self) -> Vec<usize> {
        (0..WIDTH).filter(|&c| self.can_play(c)).collect()
    }

    /// Plays a piece in the given column.
    ///
    /// After play(), `position` holds the *new* current player's perspective,
    /// so to check if the *previous* player just won, use `is_winning()`.
    pub fn play(&mut self, col: usize) -> Result<(), MoveError> {
        if col >= WIDTH {
            return Err(MoveError::OutOfRange);
        }
        if !self.can_play(col) {
            return Err(MoveError::ColumnFull);
        }
        // The lowest empty bit in this column: (mask + bottom) gives us the bit.
        self.position ^= self.mask;
        self.mask |= self.mask + bottom_mask(col);
        self.current = self.current.other();
        self.moves += 1;
        Ok(())
    }

    /// Returns true if the player who just moved has four in a row.
    ///
    /// Must be called *after* `play()` — checks the *previous* player's pieces
    /// (i.e. `position ^ mask`, since `play()` swapped perspective).
    pub fn is_winning(&self) -> bool {
        let opponent = self.position ^ self.mask;
        Self::has_alignment(opponent)
    }

    /// Returns true if the given player has won the game.
    pub fn has_won(&self, player: Player) -> bool {
        let pieces = if player == self.current {
            self.position
        } else {
            self.position ^ self.mask
        };
        Self::has_alignment(pieces)
    }

    /// Returns true if the board is completely full (draw).
    pub fn is_draw(&self) -> bool {
        self.moves as usize >= WIDTH * HEIGHT
    }

    /// Returns which player (if any) has a piece at the given (col, row) position.
    /// Row 0 is the bottom row.
    pub fn piece_at(&self, col: usize, row: usize) -> Option<Player> {
        if col >= WIDTH || row >= HEIGHT {
            return None;
        }
        let bit = 1u64 << (col * (HEIGHT + 1) + row);
        if (self.mask & bit) == 0 {
            return None;
        }
        // Determine which player owns this piece.
        // `position` is from the current player's perspective.
        let is_current = (self.position & bit) != 0;
        if is_current {
            Some(self.current)
        } else {
            Some(self.current.other())
        }
    }

    /// Check if a bitmask has four in a row in any direction.
    fn has_alignment(pos: u64) -> bool {
        // Horizontal (shift by HEIGHT+1 = 7)
        let m = pos & (pos >> (HEIGHT + 1));
        if (m & (m >> (2 * (HEIGHT + 1)))) != 0 {
            return true;
        }
        // Vertical (shift by 1)
        let m = pos & (pos >> 1);
        if (m & (m >> 2)) != 0 {
            return true;
        }
        // Diagonal \ (shift by HEIGHT = 6)
        let m = pos & (pos >> HEIGHT);
        if (m & (m >> (2 * HEIGHT))) != 0 {
            return true;
        }
        // Diagonal / (shift by HEIGHT+2 = 8)
        let m = pos & (pos >> (HEIGHT + 2));
        if (m & (m >> (2 * (HEIGHT + 2)))) != 0 {
            return true;
        }
        false
    }
}

/// Bottom bit of a column.
fn bottom_mask(col: usize) -> u64 {
    1u64 << (col * (HEIGHT + 1))
}

/// Top playable bit of a column.
fn top_mask(col: usize) -> u64 {
    1u64 << (col * (HEIGHT + 1) + HEIGHT - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_board_is_empty() {
        let b = Bitboard::new();
        assert_eq!(b.current_player(), Player::Red);
        assert_eq!(b.move_count(), 0);
        assert!(!b.is_draw());
        for col in 0..WIDTH {
            assert!(b.can_play(col));
            for row in 0..HEIGHT {
                assert_eq!(b.piece_at(col, row), None);
            }
        }
    }

    #[test]
    fn play_and_piece_at() {
        let mut b = Bitboard::new();
        b.play(3).unwrap();
        assert_eq!(b.piece_at(3, 0), Some(Player::Red));
        assert_eq!(b.current_player(), Player::Yellow);
        b.play(3).unwrap();
        assert_eq!(b.piece_at(3, 1), Some(Player::Yellow));
        assert_eq!(b.current_player(), Player::Red);
    }

    #[test]
    fn vertical_win() {
        let mut b = Bitboard::new();
        // Red: col 0, Yellow: col 1, alternating
        for _ in 0..3 {
            b.play(0).unwrap(); // Red
            b.play(1).unwrap(); // Yellow
        }
        b.play(0).unwrap(); // Red — 4th in col 0
        assert!(b.is_winning());
        assert!(b.has_won(Player::Red));
        assert!(!b.has_won(Player::Yellow));
    }

    #[test]
    fn horizontal_win() {
        let mut b = Bitboard::new();
        // Red plays cols 0-3 on row 0, Yellow plays same cols on row 1 (offset by one move)
        // R:0, Y:0, R:1, Y:1, R:2, Y:2, R:3 — Red has bottom row 0,1,2,3
        b.play(0).unwrap(); // R
        b.play(0).unwrap(); // Y
        b.play(1).unwrap(); // R
        b.play(1).unwrap(); // Y
        b.play(2).unwrap(); // R
        b.play(2).unwrap(); // Y
        b.play(3).unwrap(); // R — horizontal win
        assert!(b.is_winning());
        assert!(b.has_won(Player::Red));
    }

    #[test]
    fn diagonal_up_right_win() {
        let mut b = Bitboard::new();
        // Build a / diagonal for Red at (0,0),(1,1),(2,2),(3,3)
        // Col 0: R
        b.play(0).unwrap(); // R at (0,0)
        // Col 1: Y, R
        b.play(1).unwrap(); // Y at (1,0)
        b.play(1).unwrap(); // R at (1,1)
        // Col 2: Y, Y, R
        b.play(2).unwrap(); // R at (2,0) — wait, need Yellow fillers
        // Let me redo this more carefully.
        let mut b = Bitboard::new();
        // We need Red at (0,0), (1,1), (2,2), (3,3)
        // Col 0: [R]
        // Col 1: [Y, R]
        // Col 2: [Y, Y, R] — need extra Yellow in col 2
        // Col 3: [Y, Y, Y, R] — need extra Yellow in col 3

        b.play(0).unwrap(); // R at (0,0)
        b.play(1).unwrap(); // Y at (1,0)
        b.play(1).unwrap(); // R at (1,1)
        b.play(2).unwrap(); // Y at (2,0)
        b.play(3).unwrap(); // R at (3,0)  — filler
        b.play(2).unwrap(); // Y at (2,1)
        b.play(2).unwrap(); // R at (2,2)
        b.play(3).unwrap(); // Y at (3,1)
        b.play(4).unwrap(); // R at (4,0)  — filler
        b.play(3).unwrap(); // Y at (3,2)
        b.play(3).unwrap(); // R at (3,3)  — diagonal /!
        assert!(b.is_winning());
        assert!(b.has_won(Player::Red));
    }

    #[test]
    fn diagonal_down_right_win() {
        let mut b = Bitboard::new();
        // Build a \ diagonal for Red at (0,3),(1,2),(2,1),(3,0)
        // Col 0: [Y, Y, Y, R]
        // Col 1: [Y, Y, R]
        // Col 2: [Y, R]
        // Col 3: [R]

        // Fill col 0 with 3 yellows then red on top
        b.play(3).unwrap(); // R at (3,0)
        b.play(0).unwrap(); // Y at (0,0)
        b.play(2).unwrap(); // R at (2,0) — filler
        b.play(0).unwrap(); // Y at (0,1)
        b.play(4).unwrap(); // R at (4,0) — filler
        b.play(0).unwrap(); // Y at (0,2)
        b.play(0).unwrap(); // R at (0,3)
        b.play(1).unwrap(); // Y at (1,0)
        b.play(5).unwrap(); // R at (5,0) — filler
        b.play(1).unwrap(); // Y at (1,1)
        b.play(1).unwrap(); // R at (1,2)
        b.play(2).unwrap(); // Y at (2,1)
        b.play(2).unwrap(); // R at (2,2) — wait, need (2,1) to be R
        // This is getting messy, let me plan it properly.

        let mut b = Bitboard::new();
        // Target: Red at (3,0), (2,1), (1,2), (0,3) — a \ diagonal
        // Col 3: [R] — just Red at bottom
        // Col 2: [_, R] — need 1 filler then Red
        // Col 1: [_, _, R] — need 2 fillers then Red
        // Col 0: [_, _, _, R] — need 3 fillers then Red

        // Moves (R=Red, Y=Yellow):
        b.play(3).unwrap(); // R(3,0) — target
        b.play(2).unwrap(); // Y(2,0) — filler
        b.play(2).unwrap(); // R(2,1) — target
        b.play(1).unwrap(); // Y(1,0) — filler
        b.play(1).unwrap(); // R(1,1) — filler for col 1
        b.play(0).unwrap(); // Y(0,0) — filler
        b.play(0).unwrap(); // R(0,1) — filler for col 0
        b.play(1).unwrap(); // Y(1,2) — oops, this gives Yellow at (1,2), not Red
        // Let me think again...

        // It's tricky because Yellow is placing fillers too.
        // Let's use a different approach: just play many moves and verify.
        let mut b = Bitboard::new();
        // Use columns 4,5,6 as dumping ground for "waste" moves.
        // Red targets: (0,3),(1,2),(2,1),(3,0)
        b.play(3).unwrap(); // R(3,0)
        b.play(4).unwrap(); // Y(4,0)
        b.play(2).unwrap(); // R(2,0)
        b.play(1).unwrap(); // Y(1,0)
        b.play(2).unwrap(); // R(2,1) — target
        b.play(0).unwrap(); // Y(0,0)
        b.play(1).unwrap(); // R(1,1)
        b.play(0).unwrap(); // Y(0,1)
        b.play(1).unwrap(); // R(1,2) — target
        b.play(0).unwrap(); // Y(0,2)
        b.play(0).unwrap(); // R(0,3) — target! diagonal \
        assert!(b.is_winning());
        assert!(b.has_won(Player::Red));
    }

    #[test]
    fn column_full() {
        let mut b = Bitboard::new();
        for _ in 0..HEIGHT {
            b.play(0).unwrap();
        }
        assert!(!b.can_play(0));
        assert_eq!(b.play(0), Err(MoveError::ColumnFull));
    }

    #[test]
    fn out_of_range() {
        let mut b = Bitboard::new();
        assert_eq!(b.play(7), Err(MoveError::OutOfRange));
        assert_eq!(b.play(100), Err(MoveError::OutOfRange));
        assert!(!b.can_play(7));
    }

    #[test]
    fn draw() {
        // Test the draw detection mechanic by setting moves to the full count.
        let mut b = Bitboard::new();
        b.moves = (WIDTH * HEIGHT) as u32;
        assert!(b.is_draw());

        // A fresh board is not a draw.
        let b2 = Bitboard::new();
        assert!(!b2.is_draw());
    }

    #[test]
    fn no_false_win_on_empty() {
        let b = Bitboard::new();
        assert!(!b.is_winning());
        assert!(!b.has_won(Player::Red));
        assert!(!b.has_won(Player::Yellow));
    }

    #[test]
    fn valid_columns_full_board() {
        let mut b = Bitboard::new();
        // Fill only column 0
        for _ in 0..HEIGHT {
            b.play(0).unwrap();
        }
        let valid = b.valid_columns();
        assert!(!valid.contains(&0));
        assert_eq!(valid.len(), WIDTH - 1);
    }
}
