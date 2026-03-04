use c4_solver::display;
use c4_solver::game::Game;
use c4_solver::player::{ComputerPlayer, Difficulty, HumanPlayer};
use std::io::{self, BufRead, Write};
use std::time::Duration;

struct Config {
    timeout: Duration,
}

fn parse_args() -> Config {
    let args: Vec<String> = std::env::args().collect();
    let mut timeout = Duration::from_secs(5);

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!("Usage: c4-solver [OPTIONS]");
                println!();
                println!("Options:");
                println!("  --timeout <seconds>  Set solver time limit (default: 5.0)");
                println!("  -h, --help           Show this help message");
                std::process::exit(0);
            }
            "--timeout" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --timeout requires a value");
                    std::process::exit(1);
                }
                match args[i].parse::<f64>() {
                    Ok(secs) if secs > 0.0 => {
                        timeout = Duration::from_secs_f64(secs);
                    }
                    _ => {
                        eprintln!("Error: --timeout must be a positive number");
                        std::process::exit(1);
                    }
                }
            }
            other => {
                eprintln!("Error: unknown option '{}'", other);
                eprintln!("Try '--help' for more information.");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    Config { timeout }
}

fn main() {
    let config = parse_args();

    display::print_welcome();

    loop {
        let game_mode = select_mode();
        let mut game = match game_mode {
            Mode::HumanVsHuman => Game::new(Box::new(HumanPlayer), Box::new(HumanPlayer)),
            Mode::HumanVsComputer => {
                let difficulty = select_difficulty();
                let human_color = select_color();
                let computer = ComputerPlayer::new(difficulty, config.timeout);
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
