#![feature(test)]
extern crate test;

mod board;
mod game;
mod policies;
mod small_board;

use rand::prelude::*;
use sigmazero::game::Game;
use sigmazero::mcts::MCTS;
use sigmazero::policy::Agent;

use ego_tree::NodeId;
use game::{XOGame, XOGameStatus, XOPlayer};
use policies::RandomAgent;


fn main() {
    let mut rng = rand::thread_rng();
    let mut root_game = XOGame::default();
    loop {
        if matches!(
            root_game.status(),
            XOGameStatus::InProgress {
                player: XOPlayer::O
            }
        ) {
            _ = root_game.take_turn(root_game.valid_moves().choose(&mut rng).unwrap());
        }
        let mut mcts =
            MCTS::<XOGame, RandomAgent>::from_root_game_state(root_game.clone(), RandomAgent::new());
        // println!("{}", mcts.tree);
        for _ in 0..1000 {
            let node_chain: Vec<NodeId> = mcts.select();
            let value = mcts.expand(node_chain.last().copied().unwrap());
            mcts.backup(node_chain, value);
        }

        // for child in mcts.tree.root().children() {
        //     println!("{} has N={}", child.value().previous_action.clone().map_or("Root".to_string(), |p| p.to_string()), child.value().num_visits)
        // }

        let best_child = mcts.select_best_child();
        println!("{}\n", best_child.game_state.board());

        root_game = best_child.game_state.clone();
        if best_child.is_terminal() {
            println!("Result: {:?}", best_child.game_state.status());
            break;
        }
    }
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
