use crate::game::Game;

pub struct ReplayBuffer<G> {
    games: Vec<G>,
    values: Vec<f32>,
    policies: Vec<[f32; 81]>
}