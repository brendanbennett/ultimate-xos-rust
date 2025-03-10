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

fn generate_new_games() {
    let device = tch::Device::Cpu;
    // Generate random games for initial data
    let rng = rand::thread_rng();
    let mut agent = RandomAgent { rng };

    let n_games = 1000;

    let replay = self_play(&mut agent, n_games, 800, false);
    let replay_augmented = replay.augmented();

    println!("Replay augmented from {} to {}", replay.len(), replay_augmented.len());
    // for i in 0..8 {
    //     let print_idx = i+64;
    //     println!("{}\n{}\n{}", replay_augmented.games[print_idx], XOGame::displays(replay_augmented.policies[print_idx].format_to_print()), replay_augmented.values[print_idx]);
    // }

    let replay_data: ReplayBufferTensorData = replay_augmented.into();
    replay_data.save_to_file(Path::new("random_games_2.ot")).unwrap();
}

fn main() {
    let device = tch::Device::Cpu;
    
    // Train NN
    let mut replay_data = ReplayBufferTensorData::load_from_file(Path::new("random_games_2.ot"), device).unwrap();
    println!("Cuda available: {:?}", tch::Cuda::is_available());
    println!("Cudnn available: {}", tch::Cuda::cudnn_is_available());
    
    let batch_size = 32;
    let epochs = 100;
    let vs = nn::VarStore::new(device);
    train_on_replay::<XONNAgent, XOGame, 81>(&vs, &replay_data, batch_size, epochs, 0.8);
    vs.save("model_0.ot".to_string()).expect("Save Failed");

    // evaluation
    let rng = rand::thread_rng();
    let mut agent1 = RandomAgent { rng };

    let mut vs = nn::VarStore::new(device);
    vs.load(&Path::new("./model_0.ot"))
        .expect("Model load failed");
    let mut agent2 = XONNAgent::new(&vs);

    let evaluation_results = evaluate_agents(&mut agent1, &mut agent2, 40, 400, false);
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

#[test]
fn test_model_save() {
    let device = tch::Device::Cpu;
    let game = XOGame::default();
    let vs1 = nn::VarStore::new(device);
    let mut agent1 = XONNAgent::new(&vs1);
    let eval1 = agent1.eval_game(&game);
    vs1.save("model_test.ot".to_string()).expect("Save Failed");

    let mut vs2 = nn::VarStore::new(device);
    vs2.load(&Path::new("./model_test.ot"))
        .expect("Model load failed");
    let mut agent2 = XONNAgent::new(&vs2);
    let eval2 = agent2.eval_game(&game);

    assert_eq!(eval1, eval2);
}
