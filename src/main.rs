#![feature(test)]
extern crate test;

mod board;
mod small_board;
mod game;

use board::Position;
use rand::seq::SliceRandom;
use rand::prelude::*;
use itertools::Itertools;
use game::{Game, GameStatus};
use ego_tree::{NodeId, NodeMut, Tree};

struct GameNode {
    num_visits: u32,
    prior_prob: f32,
    total_value: f32,
    game_state: Game,
    previous_action: Option<Position>, // Only None for root
    num_children: u32
}

impl GameNode {
    fn new(prior_prob: f32, game_state: Game, previous_action: Option<Position>) -> Self {
        Self {
            num_visits: 0,
            prior_prob,
            total_value: 0.0,
            game_state,
            previous_action,
            num_children: 0,
        }
    }

    fn is_terminal(&self) -> bool {
        !matches!(self.game_state.status(), GameStatus::InProgress { player: _ })
    }
}



type MCTSTree = Tree<GameNode>;

struct MCTS {
    tree: MCTSTree,
}

impl MCTS {
    fn expand(&mut self, leaf_node_id: NodeId) -> f32 {
        let mut leaf_node: NodeMut<'_, GameNode> = self.tree.get_mut(leaf_node_id).unwrap();
        if leaf_node.value().is_terminal() {
            let status=  leaf_node.value().game_state.status();
            // Because can only win on your own move
            return status.clone().into()
        } else {
            if leaf_node.value().num_children != 0 {panic!("leaf node already has children! (Probably already expanded)")};
            let valid_moves = leaf_node.value().game_state.valid_moves();

            // Would be calculated from NN
            let prior_probs: Vec<f32> = vec![0.0; valid_moves.len()];
            let value: f32 = 0.0;

            leaf_node.value().num_children += valid_moves.len() as u32;

            for (valid_move, prior_prob) in valid_moves.iter().zip(prior_probs.iter()) {
                let mut child_state = leaf_node.value().game_state.clone();
                let child_status = child_state.take_turn(&valid_move);
                // init node, evaluate children an take mean of values as action value
                let child_node = GameNode::new(*prior_prob, child_state.clone(), Some(valid_move.clone()));
                
                leaf_node.append(child_node);
            };

            return value
        }
    }
    
    fn select(&self) -> NodeId {
        
    }
}

impl Default for MCTS {
    fn default() -> Self {
        Self {
            tree: MCTSTree::new(
                GameNode::new(
                    0.0, 
                    Game::default(),
                    None,
                )
            )
        }
    }
}


fn main() {
    let mut outcomes: Vec<String> = Vec::new();

    for _ in 0..100 {
        let mut game = Game::default();
        let mut rng = SmallRng::from_entropy();

        outcomes.push(loop {
            let mv = game.valid_moves().choose(&mut rng).expect("No valid moves but game not drawn (!?)").clone();
            let status = game.take_turn(&mv).unwrap();

            // println!("{}", game.board());
            // println!("{}", game.valid_moves());
            match status {
                GameStatus::Won { player: winner } => {break winner.to_string();},
                GameStatus::Draw => {break "Draw".to_string();},
                GameStatus::InProgress { player: _ } => (),
            }
        })
    }

    let outcome_counts= outcomes.into_iter().counts();
    println!("X: {}, O: {}, Draw: {}", outcome_counts["X"], outcome_counts["O"], outcome_counts["Draw"]);
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
