use sigmazero::policy::{Agent, Policy, RawPolicy};
use sigmazero::game::Game;
use crate::game::XOGame;
use rand::prelude::*;

pub struct RandomAgent<R: Rng> {
    pub rng: R,
}

impl<R: Rng> Agent<XOGame, { XOGame::MAX_ACTIONS }> for RandomAgent<R> {
    fn eval(&self, game: &XOGame) -> (Policy<XOGame, { XOGame::MAX_ACTIONS }>, f32) {
        (RawPolicy::new([1.0; XOGame::MAX_ACTIONS]).mask_policy(game), (rand::random::<f32>() - 0.5) * 0.2)
    }
}
