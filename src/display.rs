use crate::bitboard::{Bitboard, Player, HEIGHT, WIDTH};

const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

/// Prints the board to stdout with ANSI-colored pieces.
pub fn print_board(board: &Bitboard) {
    println!();
    // Print column headers
    print!(" ");
    for col in 0..WIDTH {
        print!(" {} ", col + 1);
    }
    println!();

    // Print rows top-to-bottom (row 5 at top, row 0 at bottom)
    for row in (0..HEIGHT).rev() {
        print!(" ");
        for col in 0..WIDTH {
            match board.piece_at(col, row) {
                Some(Player::Red) => print!("{RED} \u{25cf} {RESET}"),
                Some(Player::Yellow) => print!("{YELLOW} \u{25cf} {RESET}"),
                None => print!(" \u{00b7} "),
            }
        }
        println!();
    }
    // Bottom border
    print!(" ");
    for _ in 0..WIDTH {
        print!("---");
    }
    println!();
}

/// Prints a prompt for the current player's turn.
pub fn print_turn(player: Player) {
    let (color, name) = player_style(player);
    print!("{color}{BOLD}{name}{RESET}'s turn. Enter column (1-7): ");
    // Flush stdout so the prompt appears before reading input.
    use std::io::Write;
    std::io::stdout().flush().ok();
}

/// Prints the game result.
pub fn print_result(board: &Bitboard) {
    print_board(board);
    if board.is_winning() {
        // The winner is the player who just moved (i.e. the *previous* player).
        let winner = board.current_player().other();
        let (color, name) = player_style(winner);
        println!("{color}{BOLD}{name} wins!{RESET}");
    } else {
        println!("{BOLD}It's a draw!{RESET}");
    }
    println!();
}

/// Prints invalid input feedback.
pub fn print_invalid_input(msg: &str) {
    println!("  Invalid input: {msg}");
}

/// Prints a welcome banner.
pub fn print_welcome() {
    println!("{BOLD}=== Connect Four ==={RESET}");
    println!();
}

fn player_style(player: Player) -> (&'static str, &'static str) {
    match player {
        Player::Red => (RED, "Red"),
        Player::Yellow => (YELLOW, "Yellow"),
    }
}
