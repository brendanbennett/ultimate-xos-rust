use game::Game;
use tch::{self, nn, Tensor};
use rand::prelude::*;

use crate::{board::PositionList, game::Game};

pub struct Policy {
    moves: PositionList,
    probabilities: Vec<f32>,
}

pub struct RawPolicy([f32; 81]);

pub trait Agent {
    fn eval(game: Game) -> (Policy, f32);
}

pub struct RandomAgent;

impl Agent for RandomAgent {
    fn eval(game: Game) -> (Policy, f32) {
        (mask_policy([1.0; 81], &game), (rand.random() - 0.5) * 0.2)
    }
}

pub fn mask_policy(raw_policy: RawPolicy, game: &Game) -> Policy {
    let moves = game.valid_moves();
    let mut probabilities: Vec<f32> = moves.iter().map(|p| raw_policy[p.into()]);
    let sum = probabilities.sum();
    probabilities = probabilities.iter().map(|p| p / sum);
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