use crate::bitboard::Bitboard;
use crate::display;
use rand::seq::SliceRandom;
use std::io::{self, BufRead};

/// A controller that chooses which column to play.
pub trait PlayerController {
    fn choose_column(&self, board: &Bitboard) -> usize;
}

/// Human player: reads column choice from stdin (1-indexed).
pub struct HumanPlayer;

impl PlayerController for HumanPlayer {
    fn choose_column(&self, board: &Bitboard) -> usize {
        let stdin = io::stdin();
        loop {
            display::print_turn(board.current_player());
            let mut line = String::new();
            if stdin.lock().read_line(&mut line).is_err() || line.trim().is_empty() {
                display::print_invalid_input("please enter a number 1-7.");
                continue;
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

/// Computer player: picks a random valid column.
pub struct RandomPlayer;

impl PlayerController for RandomPlayer {
    fn choose_column(&self, board: &Bitboard) -> usize {
        let valid = board.valid_columns();
        let mut rng = rand::thread_rng();
        *valid.choose(&mut rng).expect("no valid columns")
    }
}
