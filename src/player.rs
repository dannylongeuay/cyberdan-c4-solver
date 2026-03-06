use crate::bitboard::Bitboard;
use crate::display;
use crate::solver;
use std::io::{self, BufRead};
use std::time::Duration;

/// A controller that chooses which column to play.
pub trait PlayerController {
    fn choose_column(&self, board: &Bitboard) -> usize;
    fn is_human(&self) -> bool;
}

/// Human player: reads column choice from stdin (1-indexed).
pub struct HumanPlayer;

impl PlayerController for HumanPlayer {
    fn is_human(&self) -> bool {
        true
    }

    fn choose_column(&self, board: &Bitboard) -> usize {
        let stdin = io::stdin();
        loop {
            display::print_turn(board.current_player());
            let mut line = String::new();
            match stdin.lock().read_line(&mut line) {
                Ok(0) => std::process::exit(0),
                Err(_) => {
                    display::print_invalid_input("please enter a number 1-7.");
                    continue;
                }
                Ok(_) if line.trim().is_empty() => {
                    display::print_invalid_input("please enter a number 1-7.");
                    continue;
                }
                Ok(_) => {}
            }
            match line.trim().parse::<usize>() {
                Ok(n) if (1..=7).contains(&n) => {
                    let col = n - 1; // convert to 0-indexed
                    if board.can_play(col) {
                        return col;
                    }
                    display::print_invalid_input("that column is full.");
                }
                Ok(_) => display::print_invalid_input("please enter a number 1-7."),
                Err(_) => display::print_invalid_input("please enter a number 1-7."),
            }
        }
    }
}

/// Difficulty levels for the computer player.
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    /// Returns the search depth for this difficulty level.
    pub fn depth(self) -> u32 {
        match self {
            Difficulty::Easy => 3,
            Difficulty::Normal => 9,
            Difficulty::Hard => 18,
        }
    }
}

/// Computer player: uses negamax solver with configurable difficulty.
pub struct ComputerPlayer {
    depth: u32,
    timeout: Duration,
}

impl ComputerPlayer {
    pub fn new(difficulty: Difficulty, timeout: Duration) -> Self {
        ComputerPlayer {
            depth: difficulty.depth(),
            timeout,
        }
    }
}

impl PlayerController for ComputerPlayer {
    fn is_human(&self) -> bool {
        false
    }

    fn choose_column(&self, board: &Bitboard) -> usize {
        solver::best_move(board, self.depth, self.timeout)
    }
}
