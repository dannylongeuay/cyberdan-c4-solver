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

impl Default for Bitboard {
    fn default() -> Self {
        Self::new()
    }
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

    /// Returns the current player's piece bitmask.
    pub fn position_mask(&self) -> u64 {
        self.position
    }

    /// Returns the bitmask of all pieces on the board.
    pub fn all_mask(&self) -> u64 {
        self.mask
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

    /// Parses a visual ASCII board into a `Bitboard`.
    ///
    /// Tokens per cell: `R` = Red, `Y` = Yellow, `·` or `.` = empty.
    /// Lines of digits (column headers) or dashes (footer) are skipped.
    /// The first content row is the top of the board (row 5), last is the bottom (row 0).
    ///
    /// # Panics
    /// Panics on unrecognised tokens or wrong number of columns in a row.
    pub fn from_ascii(s: &str, to_move: Player) -> Bitboard {
        let mut position: u64 = 0;
        let mut mask: u64 = 0;
        let mut moves: u32 = 0;
        let mut row_index = HEIGHT;

        for line in s.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.chars().all(|c| c.is_ascii_digit() || c.is_whitespace()) {
                continue;
            }
            if trimmed.chars().all(|c| c == '-' || c.is_whitespace()) {
                continue;
            }

            assert!(row_index > 0, "too many content rows");
            row_index -= 1;

            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            assert_eq!(
                tokens.len(),
                WIDTH,
                "expected {WIDTH} tokens, got {}",
                tokens.len()
            );

            for (col, token) in tokens.iter().enumerate() {
                match *token {
                    "R" | "Y" => {
                        let bit = 1u64 << (col * (HEIGHT + 1) + row_index);
                        mask |= bit;
                        if (*token == "R") == (to_move == Player::Red) {
                            position |= bit;
                        }
                        moves += 1;
                    }
                    "\u{00b7}" | "." => {}
                    other => panic!("unexpected token: {other:?}"),
                }
            }
        }

        Bitboard {
            position,
            mask,
            current: to_move,
            moves,
        }
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
        let b = Bitboard::from_ascii(
            "
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             R  .  .  .  .  .  .
             R  Y  .  .  .  .  .
             R  Y  .  .  .  .  .
             R  Y  .  .  .  .  .
            ",
            Player::Yellow,
        );
        assert!(b.is_winning());
        assert!(b.has_won(Player::Red));
        assert!(!b.has_won(Player::Yellow));
    }

    #[test]
    fn horizontal_win() {
        let b = Bitboard::from_ascii(
            "
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             Y  Y  Y  .  .  .  .
             R  R  R  R  .  .  .
            ",
            Player::Yellow,
        );
        assert!(b.is_winning());
        assert!(b.has_won(Player::Red));
    }

    #[test]
    fn diagonal_up_right_win() {
        // Red / diagonal at (0,0),(1,1),(2,2),(3,3)
        let b = Bitboard::from_ascii(
            "
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  R  .  .  .
             .  .  R  Y  .  .  .
             .  R  Y  Y  .  .  .
             R  Y  Y  R  R  .  .
            ",
            Player::Yellow,
        );
        assert!(b.is_winning());
        assert!(b.has_won(Player::Red));
    }

    #[test]
    fn diagonal_down_right_win() {
        // Red \ diagonal at (0,3),(1,2),(2,1),(3,0)
        let b = Bitboard::from_ascii(
            "
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             R  .  .  .  .  .  .
             Y  R  .  .  .  .  .
             Y  R  R  .  .  .  .
             Y  Y  R  R  Y  .  .
            ",
            Player::Yellow,
        );
        assert!(b.is_winning());
        assert!(b.has_won(Player::Red));
    }

    #[test]
    fn column_full() {
        let mut b = Bitboard::from_ascii(
            "
             Y  .  .  .  .  .  .
             R  .  .  .  .  .  .
             Y  .  .  .  .  .  .
             R  .  .  .  .  .  .
             Y  .  .  .  .  .  .
             R  .  .  .  .  .  .
            ",
            Player::Red,
        );
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
        // A valid 42-move draw position (2x2 block pattern, no 4-in-a-row).
        let b = Bitboard::from_ascii(
            "
             Y  Y  R  R  Y  Y  R
             R  R  Y  Y  R  R  Y
             Y  Y  R  R  Y  Y  R
             R  R  Y  Y  R  R  Y
             Y  Y  R  R  Y  Y  R
             R  R  Y  Y  R  R  Y
            ",
            Player::Red,
        );
        assert!(b.is_draw());
        assert!(!b.has_won(Player::Red));
        assert!(!b.has_won(Player::Yellow));

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
        let b = Bitboard::from_ascii(
            "
             Y  .  .  .  .  .  .
             R  .  .  .  .  .  .
             Y  .  .  .  .  .  .
             R  .  .  .  .  .  .
             Y  .  .  .  .  .  .
             R  .  .  .  .  .  .
            ",
            Player::Red,
        );
        let valid = b.valid_columns();
        assert!(!valid.contains(&0));
        assert_eq!(valid.len(), WIDTH - 1);
    }

    #[test]
    fn from_ascii_empty_board() {
        let b = Bitboard::from_ascii(
            "
             1  2  3  4  5  6  7
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             ---------------------
            ",
            Player::Red,
        );
        assert_eq!(b, Bitboard::new());
    }

    #[test]
    fn from_ascii_single_piece() {
        let b = Bitboard::from_ascii(
            "
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  R  .  .  .
            ",
            Player::Yellow,
        );
        assert_eq!(b.piece_at(3, 0), Some(Player::Red));
        assert_eq!(b.move_count(), 1);
        assert_eq!(b.current_player(), Player::Yellow);
    }

    #[test]
    fn from_ascii_round_trip() {
        // Build a board via play() calls
        let mut expected = Bitboard::new();
        expected.play(3).unwrap(); // R at (3,0)
        expected.play(3).unwrap(); // Y at (3,1)
        expected.play(2).unwrap(); // R at (2,0)

        // Parse the equivalent ASCII (Yellow to move, 3 pieces placed)
        let parsed = Bitboard::from_ascii(
            "
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  .  .  .  .
             .  .  .  Y  .  .  .
             .  .  R  R  .  .  .
            ",
            Player::Yellow,
        );

        assert_eq!(parsed.position_mask(), expected.position_mask());
        assert_eq!(parsed.all_mask(), expected.all_mask());
        assert_eq!(parsed.move_count(), expected.move_count());
        assert_eq!(parsed.current_player(), expected.current_player());
    }
}
