use crate::{game::Game, policy::RawPolicy};

pub struct ReplayBuffer<G: Game<N>, const N: usize> {
    games: Vec<G>,
    values: Vec<f32>,
    policies: Vec<RawPolicy<N>>
}

impl<G: Game<N>, const N: usize> ReplayBuffer<G, N> {
    pub fn new(games: Vec<G>, values: Vec<f32>, policies: Vec<RawPolicy<N>>) -> Self {
        if !(games.len() == values.len() && values.len() == policies.len()) {
            panic!("Game, Value and Policy counts don't equal: {}, {}, {}", games.len(), values.len(), policies.len())
        }
        Self {
            games,
            values,
            policies,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&G, &f32, &RawPolicy<N>)> {
        self.games.iter()
            .zip(self.values.iter())
            .zip(self.policies.iter())
            .map(|((game, value), policy)| (game, value, policy))
    }
}