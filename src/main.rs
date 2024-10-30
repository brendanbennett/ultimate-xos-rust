#![feature(test)]
extern crate test;

mod board;
mod small_board;
mod game;

use rand::seq::SliceRandom;
use rand::prelude::*;
use itertools::Itertools;
use game::{Game, GameStatus};


fn main() {
    let mut outcomes: Vec<String> = Vec::new();

    for _ in 0..100 {
        let mut game = Game::default();
        let mut rng = SmallRng::from_entropy();

        outcomes.push(loop {
            let mv = game.valid_moves().choose(&mut rng).expect("No valid moves but game not drawn (!?)").clone();
            let status = game.take_turn(&mv).unwrap();

            // println!("{}", game.board());
            // println!("{}", game.valid_moves());
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
