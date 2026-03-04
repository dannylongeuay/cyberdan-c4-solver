use c4_solver::game::Game;
use c4_solver::player::{HumanPlayer, RandomPlayer};
use c4_solver::display;
use std::io::{self, BufRead, Write};

fn main() {
    display::print_welcome();

    loop {
        let game_mode = select_mode();
        let mut game = match game_mode {
            Mode::HumanVsHuman => Game::new(Box::new(HumanPlayer), Box::new(HumanPlayer)),
            Mode::HumanVsComputer => Game::new(Box::new(HumanPlayer), Box::new(RandomPlayer)),
        };
        game.run();

        if !ask_play_again() {
            println!("Thanks for playing!");
            break;
        }
    }
}

enum Mode {
    HumanVsHuman,
    HumanVsComputer,
}

fn select_mode() -> Mode {
    let stdin = io::stdin();
    loop {
        println!("Select mode:");
        println!("  1) Human vs Human");
        println!("  2) Human vs Computer");
        print!("Choice: ");
        io::stdout().flush().ok();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            continue;
        }
        match line.trim() {
            "1" => return Mode::HumanVsHuman,
            "2" => return Mode::HumanVsComputer,
            _ => println!("Please enter 1 or 2."),
        }
    }
}

fn ask_play_again() -> bool {
    let stdin = io::stdin();
    loop {
        print!("Play again? (y/n): ");
        io::stdout().flush().ok();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            return false;
        }
        match line.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Please enter y or n."),
        }
    }
}
