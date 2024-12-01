use std::time::Instant;

use crate::{
    data::{ReplayBuffer, ReplayBufferTensorData},
    game::Game,
    policy::NNAgent,
};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use tch::nn::{self, OptimizerConfig, VarStore};

pub fn train_on_replay<A: NNAgent<G, N>, G: Game<N>, const N: usize>(
    vs: &VarStore,
    replay_data: &ReplayBufferTensorData,
    batch_size: usize,
    epochs: usize,
    train_fraction: f32,
) {
    // Start training NN
    let mut nn_agent = A::new(&vs);
    let device= vs.device();
    if device != replay_data.device() {
        panic!("Agent device ({:?}) and replay device ({:?}) mismatch.", device, replay_data.device());
    }
    let mut opt = nn::Adam::default()
        .build(&vs, 1e-3)
        .expect("Optimiser initialisation failed!");

    let (train_data, test_data) = replay_data.random_split(train_fraction);
    let progress_bar = ProgressBar::new(epochs as u64);
    progress_bar.set_style(
        ProgressStyle::with_template("{msg}\n[{elapsed_precise}] {bar:40} {pos}/{len} epochs")
            .unwrap(),
    );

    let train_batches = tch::data::Iter2::new(
        &train_data.features,
        &train_data.policy_value,
        batch_size as i64,
    ).collect::<Vec<_>>().len();
    let mut temp_test_loader = tch::data::Iter2::new(
        &test_data.features,
        &test_data.policy_value,
        batch_size as i64,
    );
    temp_test_loader.return_smaller_last_batch();
    let test_batches = temp_test_loader.collect::<Vec<_>>().len();
    println!("Training for {} epochs on {} batches of {}...", epochs, train_batches, batch_size);
    let start = Instant::now();
    for epoch in 0..epochs {
        progress_bar.inc(1);
        let mut total_epoch_loss: [f64; 2] = [0.0, 0.0];
        let mut train_data_iterator = tch::data::Iter2::new(
            &train_data.features,
            &train_data.policy_value,
            batch_size as i64,
        );
        train_data_iterator.to_device(device);
        train_data_iterator.shuffle();
        for (features, policy_values) in train_data_iterator {
            let mut pv_split = policy_values.split_with_sizes(&[81, 1], -1);
            let value_target = pv_split.pop().unwrap();
            let policy_target = pv_split.pop().unwrap();
            let (policy_est, value_est) = nn_agent.forward(&features);

            let value_loss = value_est.mse_loss(&value_target, tch::Reduction::Mean);
            // KL-divergence for prob distributions
            let policy_loss = policy_est
                .log()
                .kl_div(&policy_target, tch::Reduction::Mean, false);

            total_epoch_loss[0] += policy_loss.double_value(&[]) as f64;
            total_epoch_loss[1] += value_loss.double_value(&[]) as f64;

            let loss = value_loss * policy_loss;

            opt.backward_step(&loss);
        }

        let mut total_epoch_loss_test: [f64; 2] = [0.0, 0.0];
        let mut test_data_iterator = tch::data::Iter2::new(
            &test_data.features,
            &test_data.policy_value,
            batch_size as i64,
        );
        test_data_iterator.to_device(device);
        test_data_iterator.return_smaller_last_batch();
        test_data_iterator.shuffle();
        for (features, policy_values) in test_data_iterator {
            let mut pv_split = policy_values.split_with_sizes(&[81, 1], -1);
            let value_target = pv_split.pop().unwrap();
            let policy_target = pv_split.pop().unwrap();
            let (policy_est, value_est) = tch::no_grad(|| nn_agent.forward(&features));

            let value_loss = value_est.mse_loss(&value_target, tch::Reduction::Mean);
            // KL-divergence for prob distributions
            let policy_loss = policy_est
                .log()
                .kl_div(&policy_target, tch::Reduction::Mean, false);

            total_epoch_loss_test[0] += policy_loss.double_value(&[]) as f64;
            total_epoch_loss_test[1] += value_loss.double_value(&[]) as f64;
        }
        progress_bar.set_message(format!(
            "Train - Policy loss: {:.4e}, Value loss: {:.4e} | Test - Policy loss: {:.4e}, Value loss: {:.4e}",
            total_epoch_loss[0] / (train_batches as f64),
            total_epoch_loss[1] / (train_batches as f64),
            total_epoch_loss_test[0] / (test_batches as f64),
            total_epoch_loss_test[1] / (test_batches as f64),
        ));
    };
    progress_bar.finish();
    let duration = start.elapsed();
    println!("Completed training in {:2}", duration.as_secs_f32());
}
