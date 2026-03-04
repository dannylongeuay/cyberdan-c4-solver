use crate::bitboard::Bitboard;
use crate::display;
use crate::player::PlayerController;

/// A Connect Four game with two player controllers.
pub struct Game {
    board: Bitboard,
    players: [Box<dyn PlayerController>; 2],
}

impl Game {
    /// Creates a new game with the given controllers.
    /// `players[0]` controls Red (moves first), `players[1]` controls Yellow.
    pub fn new(red: Box<dyn PlayerController>, yellow: Box<dyn PlayerController>) -> Self {
        Game {
            board: Bitboard::new(),
            players: [red, yellow],
        }
    }

    /// Runs the game loop until someone wins or the board is full.
    pub fn run(&mut self) {
        loop {
            display::print_board(&self.board);

            let player_idx = self.board.move_count() as usize % 2;
            let col = self.players[player_idx].choose_column(&self.board);

            // The controller should only return valid columns, but guard anyway.
            if let Err(e) = self.board.play(col) {
                display::print_invalid_input(&format!("{:?}", e));
                continue;
            }

            if self.board.is_winning() || self.board.is_draw() {
                display::print_result(&self.board);
                return;
            }
        }
    }
}
