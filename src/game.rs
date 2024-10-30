use crate::board::{MainBoard, Position, PositionList};
use crate::small_board::Player;


#[derive(Clone)]
pub enum GameStatus {
    InProgress {player: Player},
    Won {player: Player},
    Draw,
}

impl Default for GameStatus {
    fn default() -> Self {
        Self::InProgress { player: Player::X }
    }
}

#[derive(Debug)]
pub enum GameError {
    InvalidMove { position: Position },
    GameOver,
}

pub struct Game {
    board: MainBoard,
    status: GameStatus,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            board: MainBoard::default(),
            status: GameStatus::default(),
        }
    }
}

impl Game {
    pub fn take_turn(&mut self, position: &Position) -> Result<GameStatus, GameError> {
        let current_player = match self.status {
            GameStatus::InProgress {player} => player,
            _ => return Err(GameError::GameOver),
        };

        if !self.board.is_valid_move(position) {
            println!("is invalid");
            return Err(GameError::InvalidMove { position: position.clone() });
        }

        self.board.set_cell(position, current_player);

        self.status = if let Some(winner) = self.board.winner() {
            GameStatus::Won {player: winner}
        } else if self.board.is_draw() {
            GameStatus::Draw
        } else {
            GameStatus::InProgress {player: current_player.other_player()}
        };

        Ok(self.status.clone())
    }

    pub fn valid_moves(&self) -> PositionList {
        self.board.valid_moves()
    }

    pub fn status(&self) -> &GameStatus {
        &self.status
    }

    pub fn board(&self) -> &MainBoard {
        &self.board
    }
}
