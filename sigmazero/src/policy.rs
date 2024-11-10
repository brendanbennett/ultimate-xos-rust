use tch::{self, nn, Tensor};
use rand::prelude::*;

use crate::game::{Game, Position, PositionList};

pub struct Policy<G: Game> {
    moves: PositionList<G::Position>,
    probabilities: Vec<f32>,
}

impl<G: Game> IntoIterator for Policy<G> {
    type Item = (G::Position, f32);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let positions = self.moves.clone().into_iter();
        let probs = self.probabilities.into_iter();

        positions.zip(probs).collect::<Vec<_>>().into_iter()
    }
}

pub struct RawPolicy (Vec<f32>);

impl RawPolicy {
    pub fn new(vec: Vec<f32>) -> Self {
        Self(vec)
    }

    pub fn mask_policy<G: Game>(&self, game: &G) -> Policy<G> {
        let moves = game.valid_moves();
        let mut probabilities: Vec<f32> = moves.iter().map(|p| self.0[Into::<usize>::into(p.clone())]).collect();
        let sum: f32 = probabilities.iter().sum();
        probabilities = probabilities.into_iter().map(|p| p / sum).collect();
        Policy { moves, probabilities }
    }
}

pub trait Agent<G: Game> {
    fn eval(&self, game: &G) -> (Policy<G>, f32);
    fn new() -> Self;
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