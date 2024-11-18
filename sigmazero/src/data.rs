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

    pub fn append(&mut self, games: &mut Vec<G>, values: &mut Vec<f32>, policies: &mut Vec<RawPolicy<N>>) {
        if !(games.len() == values.len() && values.len() == policies.len()) {
            panic!("Game, Value and Policy counts don't equal: {}, {}, {}", games.len(), values.len(), policies.len())
        }
        self.games.append(games);
        self.values.append(values);
        self.policies.append(policies);
    }

    pub fn to_tensor_data() {

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

pub struct ReplayBufferTensorData<> {
    pub games: tch::Tensor,
    pub values: tch::Tensor,
    pub policies: tch::Tensor,
}

impl<G: Game<N>, const N: usize> From<ReplayBuffer<G, N>> for ReplayBufferTensorData {
    fn from(buffer: ReplayBuffer<G, N>) -> Self {
        Self {
            games: tch::Tensor::stack(&buffer.games.into_iter().map(|g| g.features()).collect::<Vec<_>>(), 0),
            values: tch::Tensor::from_slice(&buffer.values),
            policies: tch::Tensor::stack(&buffer.policies.into_iter().map(|p| p.to_tensor(&[N as i64])).collect::<Vec<_>>(), 0)
        }
    }
}