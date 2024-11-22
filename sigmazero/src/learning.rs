use std::time::Instant;

use crate::{
    data::{ReplayBuffer, ReplayBufferTensorData},
    game::Game,
    policy::NNAgent,
};
use indicatif::{ProgressBar, ProgressStyle};
use tch::nn::{self, OptimizerConfig, VarStore};

pub fn train_on_replay<A: NNAgent<G, N>, G: Game<N>, const N: usize>(
    vs: &VarStore,
    replay_buffer: &ReplayBuffer<G, N>,
    batch_size: usize,
    epochs: usize,
) {
    // Start training NN
    let mut nn_agent = A::new(&vs);
    let mut opt = nn::Adam::default()
        .build(&vs, 1e-3)
        .expect("Optimiser initialisation failed!");

    let train_data: ReplayBufferTensorData = replay_buffer.clone().into();
    let progress_bar = ProgressBar::new(epochs as u64);
    progress_bar.set_style(
        ProgressStyle::with_template("{msg}\n[{elapsed_precise}] {bar:40} {pos}/{len} epochs")
            .unwrap(),
    );

    let num_batches = (train_data.features.size()[0] as f32 / batch_size as f32).ceil() as usize;
    println!("Training for {} epochs on {} batches of {}...", epochs, num_batches, batch_size);
    let start = Instant::now();
    for epoch in 0..epochs {
        progress_bar.inc(1);
        let mut total_epoch_loss: [f64; 2] = [0.0, 0.0];
        let mut data_iterator = tch::data::Iter2::new(
            &train_data.features,
            &train_data.policy_value,
            batch_size as i64,
        );
        data_iterator.shuffle();
        for (features, policy_values) in data_iterator {
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

            let loss = value_loss + policy_loss;

            opt.backward_step(&loss);
        }
        progress_bar.set_message(format!(
            "Policy loss: {}, Value loss: {}",
            total_epoch_loss[0] / (num_batches as f64),
            total_epoch_loss[1] / (num_batches as f64)
        ));
    };
    progress_bar.finish();
    let duration = start.elapsed();
    println!("Completed training in {:2}", duration.as_secs_f32());
}
