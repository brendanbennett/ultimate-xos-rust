#![feature(test)]
extern crate test;

mod board;
mod game;
mod policies;
mod small_board;

use colored::Colorize;
use itertools::Itertools;
use sigmazero::policy::RawPolicy;
use sigmazero::{game::Game, mcts::self_play};
use std::mem::size_of_val;
use std::time::Instant;
use tch::nn;

use game::XOGame;
use policies::RandomAgent;

fn format_raw_policy<const N: usize>(raw_policy: &RawPolicy<N>) -> Vec<String> {
    raw_policy
        .to_vec()
        .iter()
        .map(|n| colour_number(*n))
        .collect()
}

fn colour_number(number: f32) -> String {
    let mut s = String::new();
    if number == 1.0 {
        s = "1.0".to_string();
    } else {
        s = format!("{number:3.2}")[1..].to_string();
    }
    if number > 0.25 {
        s = s.red().to_string()
    } else if number > 0.005 {
        s = s.yellow().to_string()
    }
    s
}

fn main() {
    let rng = rand::thread_rng();
    let mut agent = RandomAgent { rng };

    let n_games = 1;

    let start = Instant::now();
    let replay = self_play(&mut agent, n_games, false);
    let duration = start.elapsed();

    let vs = nn::VarStore::new(tch::Device::Cpu);

    let nn_agent = 


    // for (game, value, policy) in replay.iter() {
    //     println!("{game}");
    //     // println!("{}", game.features());
    //     println!("{}", XOGame::displays(format_raw_policy(policy)));
    //     println!("Value: {value}");
    // }

    println!(
        "generated {} Games with size {} bytes in {:?} seconds",
        n_games,
        size_of_val(&*(replay.iter().collect_vec())),
        duration
    );
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
