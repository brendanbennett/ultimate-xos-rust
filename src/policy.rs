use tch::{self, nn, Tensor};
use rand::prelude::*;

use crate::{board::{Position, PositionList}, game::Game};

pub struct Policy {
    moves: PositionList,
    probabilities: Vec<f32>,
}

impl IntoIterator for Policy {
    type Item = (Position, f32);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let positions = self.moves.clone().into_iter();
        let probs = self.probabilities.into_iter();

        positions.zip(probs).collect::<Vec<_>>().into_iter()
    }
}

pub struct RawPolicy([f32; 81]);

pub trait Agent {
    fn eval(&self, game: &Game) -> (Policy, f32);
    fn new() -> Self;
}

pub struct RandomAgent;

impl Agent for RandomAgent {
    fn eval(&self, game: &Game) -> (Policy, f32) {
        (mask_policy(RawPolicy([1.0; 81]), game), (rand::random::<f32>() - 0.5) * 0.2)
    }

    fn new() -> Self {
        Self {}
    }
}

pub fn mask_policy(raw_policy: RawPolicy, game: &Game) -> Policy {
    let moves = game.valid_moves();
    let mut probabilities: Vec<f32> = moves.iter().map(|p| raw_policy.0[Into::<usize>::into(p.clone())]).collect();
    let sum: f32 = probabilities.iter().sum();
    probabilities = probabilities.into_iter().map(|p| p / sum).collect();
    Policy { moves, probabilities }
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