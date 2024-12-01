use std::ops::Deref;

use std::path::Path;
use tch::{nn, Tensor, Device, TchError};
use colored::Colorize;

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

#[derive(Clone, Debug, PartialEq)]
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

    pub fn format_to_print(&self) -> Vec<String> {
        self
            .to_vec()
            .iter()
            .map(|n| Self::colour_number(*n))
            .collect()
    }

    fn colour_number(number: f32) -> String {
        let mut s = String::new();
        if number == 1.0 {
            s = "1.0".to_string();
        } else {
            s = format!("{number:3.2}")[1..].to_string();
        }
        if number > 0.25 {
            s = s.red().to_string()
        } else if number > 0.005 {
            s = s.yellow().to_string()
        }
        s
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

pub trait NNAgent<G: Game<N>, const N:usize>: Agent<G, N> {
    fn new(vs: &nn::VarStore) -> Self;
    fn forward(&self, xs: &Tensor, train: bool) -> (Tensor, Tensor);
}