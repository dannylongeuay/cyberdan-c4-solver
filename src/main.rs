use c4_solver::game::Game;
use c4_solver::player::{ComputerPlayer, Difficulty, HumanPlayer};
use c4_solver::display;
use std::io::{self, BufRead, Write};

fn main() {
    display::print_welcome();

    loop {
        let game_mode = select_mode();
        let mut game = match game_mode {
            Mode::HumanVsHuman => Game::new(Box::new(HumanPlayer), Box::new(HumanPlayer)),
            Mode::HumanVsComputer => {
                let difficulty = select_difficulty();
                let human_color = select_color();
                let computer = ComputerPlayer::new(difficulty);
                match human_color {
                    HumanColor::Red => Game::new(Box::new(HumanPlayer), Box::new(computer)),
                    HumanColor::Yellow => Game::new(Box::new(computer), Box::new(HumanPlayer)),
                }
            }
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

enum HumanColor {
    Red,
    Yellow,
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

fn select_difficulty() -> Difficulty {
    let stdin = io::stdin();
    loop {
        display::print_difficulty_menu();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            continue;
        }
        match line.trim() {
            "1" => return Difficulty::Easy,
            "2" => return Difficulty::Normal,
            "3" => return Difficulty::Hard,
            _ => println!("Please enter 1, 2, or 3."),
        }
    }
}

fn select_color() -> HumanColor {
    let stdin = io::stdin();
    loop {
        display::print_color_menu();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            continue;
        }
        match line.trim() {
            "1" => return HumanColor::Red,
            "2" => return HumanColor::Yellow,
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
