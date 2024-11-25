use std::path::Path;

use crate::{
    game::Game,
    policy::{self, RawPolicy},
};
use tch::{Device, IndexOp, Kind, TchError, Tensor};

#[derive(Clone, Debug)]
pub struct ReplayBuffer<G: Game<N>, const N: usize> {
    games: Vec<G>,
    values: Vec<f32>,
    policies: Vec<RawPolicy<N>>,
}

impl<G: Game<N>, const N: usize> ReplayBuffer<G, N> {
    pub fn new(games: Vec<G>, values: Vec<f32>, policies: Vec<RawPolicy<N>>) -> Self {
        if !(games.len() == values.len() && values.len() == policies.len()) {
            panic!(
                "Game, Value and Policy counts don't equal: {}, {}, {}",
                games.len(),
                values.len(),
                policies.len()
            )
        }
        Self {
            games,
            values,
            policies,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&G, &f32, &RawPolicy<N>)> {
        self.games
            .iter()
            .zip(self.values.iter())
            .zip(self.policies.iter())
            .map(|((game, value), policy)| (game, value, policy))
    }

    pub fn append(
        &mut self,
        games: &mut Vec<G>,
        values: &mut Vec<f32>,
        policies: &mut Vec<RawPolicy<N>>,
    ) {
        if !(games.len() == values.len() && values.len() == policies.len()) {
            panic!(
                "Game, Value and Policy counts don't equal: {}, {}, {}",
                games.len(),
                values.len(),
                policies.len()
            )
        }
        self.games.append(games);
        self.values.append(values);
        self.policies.append(policies);
    }

    pub fn len(&self) -> usize {
        self.games.len()
    }

    pub fn augment(&mut self) {
        for i in 0..self.games.len() {
            let (mut aug_games, mut aug_policies) = self.games[i].augmented_with_raw_policy(&self.policies[i]);
            let mut aug_values = vec![self.values[i]; aug_games.len()];
            self.append(&mut aug_games, &mut aug_values, &mut aug_policies);
        }
    }
}

impl<G: Game<N>, const N: usize> Default for ReplayBuffer<G, N> {
    fn default() -> Self {
        Self {
            games: Vec::new(),
            values: Vec::new(),
            policies: Vec::new(),
        }
    }
}

pub struct ReplayBufferTensorData {
    pub features: tch::Tensor,
    pub policy_value: tch::Tensor,
}

impl<G: Game<N>, const N: usize> From<ReplayBuffer<G, N>> for ReplayBufferTensorData {
    fn from(buffer: ReplayBuffer<G, N>) -> Self {
        let policies = tch::Tensor::stack(
            &buffer
                .policies
                .into_iter()
                .map(|p| p.to_tensor(&[N as i64]))
                .collect::<Vec<_>>(),
            0,
        )
        .to_dtype(tch::Kind::Float, false, false);
        let values = tch::Tensor::from_slice(&buffer.values)
            .reshape(&[buffer.values.len() as i64, 1])
            .to_dtype(tch::Kind::Float, false, false);
        assert_eq!(policies.size()[1], 81);
        Self {
            features: tch::Tensor::stack(
                &buffer
                    .games
                    .into_iter()
                    .map(|g| g.features())
                    .collect::<Vec<_>>(),
                0,
            )
            .to_dtype(tch::Kind::Float, false, false),
            policy_value: tch::Tensor::cat(&[policies, values], 1),
        }
    }
}

impl ReplayBufferTensorData {
    pub fn random_split(&self, fraction: f32) -> (Self, Self) {
        let n = self.features.size()[0];
        let left_split_length = (n as f32 * fraction).ceil() as i64;
        let index = Tensor::randperm(n, (Kind::Int64, Device::Cpu));
        let mut features = self.features.detach_copy();
        let mut policy_value = self.policy_value.detach_copy();
        features = features.index_select(0, &index);
        policy_value = policy_value.index_select(0, &index);
        (
            Self {
                features: features.i(..left_split_length),
                policy_value: policy_value.i(..left_split_length),
            },
            Self {
                features: features.i(left_split_length..),
                policy_value: policy_value.i(left_split_length..),
            },
        )
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), TchError> {
        Tensor::save_multi(
            &[
                ("features", &self.features),
                ("policy_value", &self.policy_value),
            ],
            path,
        )
    }

    pub fn load_from_file(path: &Path) -> Result<Self, TchError> {
        let tensors = Tensor::load_multi(path)?;
        let features = tensors
            .iter()
            .find(|(name, _)| name == "features")
            .expect(&format!("`features` tensor not found in {:?}", path))
            .1
            .shallow_clone();
        let policy_value = tensors
            .iter()
            .find(|(name, _)| name == "policy_value")
            .expect(&format!("`policy_value` tensor not found in {:?}", path))
            .1
            .shallow_clone();
        Ok(Self {
            features,
            policy_value,
        })
    }

    pub fn len(&self) -> usize {
        self.features.size()[0] as usize
    }
}
