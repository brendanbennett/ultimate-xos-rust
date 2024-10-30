#![feature(test)]
extern crate test;

mod board;
mod small_board;

use rand::seq::SliceRandom;
use rand::prelude::*;
use board::{MainBoard, Position};
use small_board::Player;
use itertools::Itertools;

#[derive(Clone)]
enum GameStatus {
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
enum GameError {
    InvalidMove { position: Position },
    GameOver,
}

struct Game {
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

    pub fn valid_moves(&self) -> Vec<Position> {
        self.board.valid_moves()
    }

    pub fn status(&self) -> &GameStatus {
        &self.status
    }

    pub fn board(&self) -> &MainBoard {
        &self.board
    }
}


fn main() {
    let mut outcomes: Vec<String> = Vec::new();

    for _ in 0..10000 {
        let mut game = Game::default();
        let mut rng = SmallRng::from_entropy();

        outcomes.push(loop {
            let mv = game.valid_moves().choose(&mut rng).expect("No valid moves but game not drawn (!?)").clone();
            let status = game.take_turn(&mv).unwrap();

            // println!("{}\n", game.board());
            match status {
                GameStatus::Won { player: winner } => {break winner.to_string();},
                GameStatus::Draw => {break "Draw".to_string();},
                GameStatus::InProgress { player: _ } => (),
            }
        })
    }

    let outcome_counts= outcomes.into_iter().counts();
    println!("X: {}, O: {}, Draw: {}", outcome_counts["X"], outcome_counts["O"], outcome_counts["Draw"]);
}

#[cfg(test)]
mod benchmarks {
    use crate::board::play_random_game;
    use test::Bencher;

    #[bench]
    fn bench_play_game(b: &mut Bencher) {
        b.iter(|| play_random_game());
    }
}
