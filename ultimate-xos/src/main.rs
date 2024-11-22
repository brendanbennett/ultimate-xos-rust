#![feature(test)]
extern crate test;

mod board;
mod game;
mod policies;
mod small_board;

use colored::Colorize;
use game::XOGame;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use sigmazero::data::ReplayBufferTensorData;
use sigmazero::evaluate::evaluate_agents;
use sigmazero::learning::train_on_replay;
use sigmazero::policy::{Agent, NNAgent, RawPolicy};
use sigmazero::{game::Game, mcts::self_play};
use std::mem::size_of_val;
use std::path::Path;
use std::time::Instant;
use tch::nn::{self, OptimizerConfig};
use tch::Kind;

use policies::{RandomAgent, XONNAgent};

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
    // Generate random games for initial data
    let rng = rand::thread_rng();
    let mut agent = RandomAgent { rng };

    let n_games = 100;

    let start = Instant::now();
    let replay = self_play(&mut agent, n_games, 400, false);
    let duration = start.elapsed();
    println!(
        "generated {} Games with size {} bytes in {:.2} seconds",
        n_games,
        size_of_val(&*(replay.iter().collect_vec())),
        duration.as_secs_f32()
    );

    // Start training NN
    let batch_size = 32;
    let epochs = 1000;
    let vs = nn::VarStore::new(tch::Device::Cpu);
    train_on_replay::<XONNAgent, XOGame, 81>(&vs, &replay, batch_size, epochs);
    vs.save("model_0.ot".to_string()).expect("Save Failed");

    let rng = rand::thread_rng();
    let mut agent1 = RandomAgent { rng };

    let mut vs = nn::VarStore::new(tch::Device::Cpu);
    vs.load(&Path::new("./model_0.ot"))
        .expect("Model load failed");
    let mut agent2 = XONNAgent::new(&vs);

    let evaluation_results = evaluate_agents(&mut agent1, &mut agent2, 50, 200, false);
    println!("{:?}", evaluation_results);
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
