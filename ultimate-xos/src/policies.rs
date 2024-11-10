use sigmazero::policy::{Agent, Policy, RawPolicy};
use sigmazero::game::Game;
use crate::game::XOGame;

pub struct RandomAgent;

impl Agent<XOGame> for RandomAgent {
    fn eval(&self, game: &XOGame) -> (Policy<XOGame>, f32) {
        (RawPolicy::new(vec![1.0; XOGame::N]).mask_policy(game), (rand::random::<f32>() - 0.5) * 0.2)
    }

    fn new() -> Self {
        Self {}
    }
}