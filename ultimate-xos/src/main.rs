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
use sigmazero::policy::{RawPolicy, Agent};
use sigmazero::{game::Game, mcts::self_play};
use tch::Kind;
use std::mem::size_of_val;
use std::time::Instant;
use tch::nn::{self, OptimizerConfig};

use policies::{NNAgent, RandomAgent};

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

    let n_games = 1;

    let start = Instant::now();
    let replay = self_play(&mut agent, n_games, 100,false);
    let duration = start.elapsed();
    println!(
        "generated {} Games with size {} bytes in {:?} seconds",
        n_games,
        size_of_val(&*(replay.iter().collect_vec())),
        duration
    );

    // Start training NN
    let batch_size = 32;
    let epochs = 1000;
    let vs = nn::VarStore::new(tch::Device::Cpu);
    let mut nn_agent = NNAgent::new(&vs);
    let mut opt = nn::Adam::default().build(&vs, 1e-3).expect("Optimiser initialisation failed!");


    let train_data: ReplayBufferTensorData = replay.clone().into();
    let progress_bar = ProgressBar::new(epochs);
    progress_bar.set_style(ProgressStyle::with_template("{msg}\n[{elapsed_precise}] {bar:40}").unwrap());

    let num_batches = (train_data.features.size()[0] as f32 / batch_size as f32).ceil() as usize;
    println!("Training on {} batches of {}", num_batches, batch_size);
    for epoch in 0..epochs {
        progress_bar.inc(1);
        let mut total_epoch_loss: f32 = 0.0;
        let mut data_iterator = tch::data::Iter2::new(&train_data.features, &train_data.policy_value, batch_size);
        data_iterator.shuffle();
        for (features, policy_values) in data_iterator {
            let mut pv_split = policy_values.split_with_sizes(&[81, 1], -1);
            let value_target = pv_split.pop().unwrap();
            let policy_target = pv_split.pop().unwrap();
            let (policy_est, value_est) = nn_agent.forward(&features);

            let value_loss = value_est.mse_loss(&value_target, tch::Reduction::Mean);
            // KL-divergence for prob distributions
            let policy_loss = (&policy_target * (&policy_target / &policy_est).log()).sum(Kind::Float);
            let loss = value_loss + policy_loss;
            println!("loss: {loss}");
            total_epoch_loss = loss.double_value(&[]) as f32;

            opt.backward_step(&loss);
        }
        progress_bar.set_message(format!("Loss: {}", total_epoch_loss/(num_batches as f32)));
    }

    // for (game, value, policy) in replay.iter() {
    //     println!("{game}");
    //     // println!("{}", game.features());
    //     println!("{}", XOGame::displays(format_raw_policy(policy)));
    //     println!("Value: {value}");
    // }
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
