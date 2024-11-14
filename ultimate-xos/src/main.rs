#![feature(test)]
extern crate test;

mod board;
mod game;
mod policies;
mod small_board;

use colored::Colorize;
use rand::prelude::*;
use sigmazero::{game::Game, mcts::self_play};
use sigmazero::mcts::MCTS;
use sigmazero::policy::{Agent, RawPolicy};
use sigmazero::data::ReplayBuffer;

use ego_tree::NodeId;
use game::{XOGame, XOGameStatus, XOPlayer};
use policies::RandomAgent;

fn format_raw_policy<const N: usize>(raw_policy: &RawPolicy<N>) -> Vec<String> {
    raw_policy
        .to_vec()
        .iter()
        .map(|n| colour_number(*n))
        .collect()
}

fn colour_number(number: f32) -> String {
    let mut s = format!("{number:3.1}");
    if number > 0.25 {
        s = s.red().to_string()
    } else if number > 0.05 {
        s = s.yellow().to_string()
    }
    s
}

fn main() {
    let rng = rand::thread_rng();
    let mut agent = RandomAgent{rng};

    let replay = self_play(&mut agent);

    for (game, value, policy) in replay.iter() {
        println!("{game}");
        println!("{}", XOGame::displays(format_raw_policy(policy)));
        println!("Value: {value}");
    }
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
