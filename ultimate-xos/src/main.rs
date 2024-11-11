#![feature(test)]
extern crate test;

mod board;
mod game;
mod policies;
mod small_board;

use rand::prelude::*;
use sigmazero::{game::Game, mcts::self_play};
use sigmazero::mcts::MCTS;
use sigmazero::policy::Agent;
use sigmazero::data::ReplayBuffer;

use ego_tree::NodeId;
use game::{XOGame, XOGameStatus, XOPlayer};
use policies::RandomAgent;

fn main() {
    let rng = rand::thread_rng();
    let agent = RandomAgent{rng};

    self_play(&agent);
}

#[cfg(test)]
mod benchmarks {
    use crate::board::play_random_game;
    use test::Bencher;

    #[bench]
    fn bench_play_game(b: &mut Bencher) {
        b.iter(|| play_random_game());
    }
}
