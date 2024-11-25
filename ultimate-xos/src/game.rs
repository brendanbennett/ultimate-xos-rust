use core::panic;
use std::fmt;

pub use crate::board::XOPlayer;
use crate::board::{BoardDisplayer, MainBoard, XOPosition, XOPositionList};
use sigmazero::{game::{Game, GameError, GameStatus}, policy::RawPolicy};

pub type XOGameStatus = GameStatus<XOPlayer>;

#[derive(Clone, Copy)]
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
    const FEATURES_SHAPE: &'static [i64] = &[3, 9, 9];
    const FEATURES_SIZE: i64 = 3 * 9 * 9;

    type Player = XOPlayer;
    type Position = XOPosition;

    fn take_turn(
        &mut self,
        position: &Self::Position,
    ) -> Result<XOGameStatus, GameError<Self::Position>> {
        let current_player = match self.status {
            GameStatus::InProgress { player } => player,
            _ => return Err(GameError::GameOver),
        };

        if !self.board.is_valid_move(position) {
            return Err(GameError::InvalidMove {
                position: *position,
            });
        }

        self.board.set_cell(position, current_player);

        self.status = if let Some(winner) = self.board.winner() {
            GameStatus::Won { player: winner }
        } else if self.board.is_draw() {
            GameStatus::Draw
        } else {
            GameStatus::InProgress {
                player: current_player.other_player(),
            }
        };

        Ok(self.status)
    }

    fn valid_moves(&self) -> XOPositionList {
        self.board.valid_moves()
    }

    fn status(&self) -> &GameStatus<XOPlayer> {
        &self.status
    }

    fn displays(items: Vec<String>) -> impl fmt::Display {
        BoardDisplayer::new(items)
    }

    fn features(&self) -> tch::Tensor {
        let current_player = match self.status {
            GameStatus::InProgress { player } => player,
            _ => panic!("Features for completed game? In esta economia?"),
        };
        tch::Tensor::from_slice(
            self.board
                .features_for_player(current_player)
                .iter()
                .flatten()
                .flatten()
                .copied()
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .reshape([3, 9, 9])
        .to_dtype(tch::Kind::Float, false, false)
    }

    fn augmented_with_raw_policy(&self, raw_policy: &RawPolicy<81>) -> (Vec<Self>, Vec<RawPolicy<81>>) {
        let aug_games = vec![self.clone()];
        
    }
}

impl fmt::Display for XOGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.board)
    }
}
