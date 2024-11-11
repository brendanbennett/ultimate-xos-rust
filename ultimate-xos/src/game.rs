use std::fmt;

use crate::board::{MainBoard, XOPosition, XOPositionList};
use sigmazero::game::{Game, GameError, GameStatus};
pub use crate::board::XOPlayer;

pub type XOGameStatus = GameStatus<XOPlayer>;

#[derive(Clone)]
pub struct XOGame {
    board: MainBoard,
    status: GameStatus<XOPlayer>,
}

impl Default for XOGame {
    fn default() -> Self {
        Self {
            board: MainBoard::default(),
            status: GameStatus::default(),
        }
    }
}

impl Game<81> for XOGame {
    type Player = XOPlayer;
    type Position = XOPosition;

    fn take_turn(&mut self, position: &Self::Position) -> Result<XOGameStatus, GameError<Self::Position>> {
        let current_player = match self.status {
            GameStatus::InProgress {player} => player,
            _ => return Err(GameError::GameOver),
        };

        if !self.board.is_valid_move(position) {
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

    fn valid_moves(&self) -> XOPositionList {
        self.board.valid_moves()
    }

    fn status(&self) -> &GameStatus<XOPlayer> {
        &self.status
    }
}

impl fmt::Display for XOGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.board)
    }
}
