use std::ops::Deref;

use tch::Tensor;

use crate::game::{Game, PositionList};

pub struct Policy<G: Game<N>, const N: usize> {
    positions: PositionList<G::Position>,
    probabilities: Vec<f32>,
}

impl<G: Game<N>, const N: usize> IntoIterator for Policy<G, N> {
    type Item = (G::Position, f32);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.positions
            .to_owned()
            .into_iter()
            .zip(self.probabilities)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

pub struct RawPolicy<const N: usize>([f32; N]);

impl<const N: usize> RawPolicy<{ N }> {
    pub fn new(arr: [f32; N]) -> Self {
        Self(arr)
    }

    pub fn mask_policy<G: Game<N>>(&self, game: &G) -> Policy<G, N> {
        let positions = game.valid_moves();
        let mut probabilities: Vec<f32> = positions
            .iter()
            .map(|p| self.0[Into::<usize>::into(*p)])
            .collect();
        let sum: f32 = probabilities.iter().sum();
        probabilities = probabilities.into_iter().map(|p| p / sum).collect();
        Policy {
            positions,
            probabilities,
        }
    }

    pub fn to_tensor(&self, shape: &[i64]) -> Tensor {
        Tensor::from_slice(&self.0).reshape(shape)
    }
}

impl<const N: usize> Deref for RawPolicy<N> {
    type Target = [f32; N];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait Agent<G: Game<N>, const N: usize> {
    fn eval_game(&mut self, game: &G) -> (RawPolicy<N>, f32);
    fn eval_features(&mut self, features: &Tensor) -> (RawPolicy<N>, f32);
}

// pub struct UltimateXONNPolicy {
//     linear_1: nn::linear,
//     linear_2: nn::linear,
//     linear_3: nn::linear,
// }

// impl UltimateXONNPolicy {
//     fn new()
// }

// impl Policy for UltimateXONNPolicy {
//     fn eval(game: Game) -> ([f32, 81], f32) {

//     }
// }
