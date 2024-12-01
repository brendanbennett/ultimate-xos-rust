use std::path::Path;
use std::sync::PoisonError;

use crate::game::XOGame;
use rand::prelude::*;
use sigmazero::game::Game;
use sigmazero::policy::{self, Agent, NNAgent, RawPolicy};
use tch::{nn, Tensor};

pub struct RandomAgent<R: Rng> {
    pub rng: R,
}

impl<R: Rng> Agent<XOGame, 81> for RandomAgent<R> {
    fn eval_game(&mut self, _: &XOGame) -> (RawPolicy<81>, f32) {
        (
            RawPolicy::new([1.0; XOGame::MAX_ACTIONS]),
            (self.rng.gen::<f32>() - 0.5) * 0.2,
        )
    }

    fn eval_features(&mut self, _: &Tensor) -> (RawPolicy<81>, f32) {
        (
            RawPolicy::new([1.0; XOGame::MAX_ACTIONS]),
            (self.rng.gen::<f32>() - 0.5) * 0.2,
        )
    }
}

pub struct XONNAgent {
    conv1: nn::Conv2D,
    fc1: nn::Linear,
    fc2: nn::Linear,
    fc3: nn::Linear,
    device: tch::Device,
}

impl NNAgent<XOGame, 81> for XONNAgent {
    fn new(vs: &nn::VarStore) -> Self {
        const OUT_SIZE: i64 = 9 * 9 + 1;
        let root = &vs.root();
        let conv1 = nn::conv2d(root, 3, 96, 3, nn::ConvConfig {
            stride: 3, .. Default::default()
        });
        let fc1 = nn::linear(root, 96*9, 512, Default::default());
        let fc2 = nn::linear(root, 512, 512, Default::default());
        let fc3 = nn::linear(root, 512, OUT_SIZE, Default::default());
        Self { conv1, fc1, fc2, fc3, device: vs.device() }
    }

    fn forward(&self, xs: &Tensor, train: bool) -> (Tensor, Tensor) {
        let xs = xs
            .apply(&self.conv1)
            .relu()
            .flat_view()
            .dropout(0.2, train)
            .apply(&self.fc1)
            .relu()
            .dropout(0.2, train)
            .apply(&self.fc2)
            .relu()
            .dropout(0.2, train)
            .apply(&self.fc3);

        let mut ts = xs.split_with_sizes(&[81, 1], -1);
        let value_logits = ts.pop().unwrap();
        let policy_logits = ts.pop().unwrap().softmax(-1, tch::Kind::Float);
        (policy_logits, value_logits)
    }
}

impl Agent<XOGame, 81> for XONNAgent {
    fn eval_game(&mut self, game: &XOGame) -> (RawPolicy<81>, f32) {
        let features = game.features().to_device(self.device);
        self.eval_features(&features)
    }

    fn eval_features(&mut self, features: &Tensor) -> (RawPolicy<81>, f32) {
        let (policy_logits, value_logits) = self.forward(&features.unsqueeze(0), false); // Reshape into a singleton batch
        // println!("policy logits: {}", policy_logits);
        let policy: Vec<f32> = policy_logits.get(0).try_into().expect("Policy conversion from tensor to vec failed!");
        let value = f32::try_from(value_logits.softmax(-1, None)).expect("Value cast into f32 failed!");
        assert_eq!(policy.len(), 81);
        let policy_arr: [f32; 81] = policy.try_into().expect("Policy conversion from vec to array failed!");

        // debugging
        // let policy_arr = [1.0; 81];

        (RawPolicy::new(policy_arr), value)
    }
}
