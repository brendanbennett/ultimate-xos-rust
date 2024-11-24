#![feature(test)]
extern crate test;

mod board;
mod game;
mod policies;
mod small_board;

use game::XOGame;
use sigmazero::data::ReplayBufferTensorData;
use sigmazero::evaluate::evaluate_agents;
use sigmazero::learning::train_on_replay;
use sigmazero::policy::{Agent, NNAgent};
use sigmazero::{game::Game, mcts::self_play};
use std::path::Path;
use std::time::Instant;
use tch::nn::{self, OptimizerConfig};
use tch::Kind;

use policies::{RandomAgent, XONNAgent};

fn main() {
    // Generate random games for initial data
    let rng = rand::thread_rng();
    let mut agent = RandomAgent { rng };

    let n_games = 1000;

    let replay = self_play(&mut agent, n_games, 400, false);
    let replay_data: ReplayBufferTensorData = replay.into();
    replay_data.save_to_file(Path::new("random_games.ot")).unwrap();
    // let replay_data = ReplayBufferTensorData::load_from_file(Path::new("random_games.ot")).unwrap();

    // Start training NN
    // let batch_size = 32;
    // let epochs = 1000;
    // let vs = nn::VarStore::new(tch::Device::Cpu);
    // train_on_replay::<XONNAgent, XOGame, 81>(&vs, &replay_data, batch_size, epochs, 0.8);
    // // vs.save("model_0.ot".to_string()).expect("Save Failed");

    // // evaluation
    // let rng = rand::thread_rng();
    // let mut agent1 = RandomAgent { rng };

    // // let mut vs = nn::VarStore::new(tch::Device::Cpu);
    // // vs.load(&Path::new("./model_0.ot"))
    // //     .expect("Model load failed");
    // let mut agent2 = XONNAgent::new(&vs);

    // let evaluation_results = evaluate_agents(&mut agent1, &mut agent2, 1, 200, false);
    // println!("{:?}", evaluation_results);
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
