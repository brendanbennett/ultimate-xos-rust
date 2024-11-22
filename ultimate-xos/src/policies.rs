use std::path::Path;
use std::sync::PoisonError;

use crate::game::XOGame;
use rand::prelude::*;
use sigmazero::game::Game;
use sigmazero::policy::{Agent, RawPolicy, NNAgent};
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
    l_1: nn::Linear,
    l_2: nn::Linear,
    l_3: nn::Linear,
    l_4: nn::Linear,
}

impl NNAgent<XOGame, 81> for XONNAgent {
    fn new(vs: &nn::VarStore) -> Self {
        const OUT_SIZE: i64 = 9 * 9 + 1;
        let root = &vs.root();
        Self {
            l_1: nn::linear(root / "l_1", XOGame::FEATURES_SIZE, 256, Default::default()),
            l_2: nn::linear(root / "l_2", 256, 128, Default::default()),
            l_3: nn::linear(root / "l_3", 128, 96, Default::default()),
            l_4: nn::linear(root / "l_4", 96, OUT_SIZE, Default::default()),
        }
    }

    fn forward(&self, xs: &Tensor) -> (Tensor, Tensor) {
        let xs = xs
            .flat_view()
            .apply(&self.l_1)
            .relu()
            .apply(&self.l_2)
            .relu()
            .apply(&self.l_3)
            .relu()
            .apply(&self.l_4);

        let mut ts = xs.split_with_sizes(&[81, 1], -1);
        let value_logits = ts.pop().unwrap();
        let policy_logits = ts.pop().unwrap().softmax(-1, tch::Kind::Float);
        (policy_logits, value_logits)
    }
}

impl Agent<XOGame, 81> for XONNAgent {
    fn eval_game(&mut self, game: &XOGame) -> (RawPolicy<81>, f32) {
        let features = game.features();
        self.eval_features(&features)
    }

    fn eval_features(&mut self, features: &Tensor) -> (RawPolicy<81>, f32) {
        let (policy_logits, value_logits) = self.forward(&features.reshape([1, -1])); // Reshape into a singleton batch
        // println!("policy logits: {}", policy_logits);
        let policy: Vec<f32> = policy_logits.get(0).try_into().expect("Policy conversion from tensor to vec failed!");
        let value = f32::try_from(value_logits.softmax(-1, None)).expect("Value cast into f32 failed!");
        assert_eq!(policy.len(), 81);
        let policy_arr: [f32; 81] = policy.try_into().expect("Policy conversion from vec to array failed!");
        (RawPolicy::new(policy_arr), value)
    }
}
