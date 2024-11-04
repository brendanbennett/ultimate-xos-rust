#![feature(test)]
extern crate test;

mod board;
mod small_board;
mod game;

use core::fmt;

use board::Position;
use rand::seq::SliceRandom;
use rand::prelude::*;
use itertools::{izip, Itertools};
use game::{Game, GameStatus};
use ego_tree::{NodeId, NodeMut, NodeRef, Tree};

struct GameNode {
    num_visits: u32,
    prior_prob: f32,
    total_value: f32,
    action_value: f32,
    game_state: Game,
    node_state: GameNodeState,
    previous_action: Option<Position>, // Only None for root
    num_children: u32
}

impl GameNode {
    fn new(prior_prob: f32, game_state: Game, node_state: GameNodeState, previous_action: Option<Position>) -> Self {
        Self {
            num_visits: 0,
            prior_prob,
            total_value: 0.0,
            action_value: 0.0,
            game_state,
            node_state,
            previous_action,
            num_children: 0,
        }
    }

    fn is_terminal(&self) -> bool {
        !matches!(self.game_state.status(), GameStatus::InProgress { player: _ })
    }

    fn update_action_value(&mut self) {
        self.action_value = self.total_value / self.num_visits as f32;
    }
}

impl fmt::Display for GameNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, ", if let Some(prev_action) = &self.previous_action {prev_action.to_string()} else {"Root".to_string()})?;
        write!(f, "{:?}, N={}", self.node_state, self.num_visits)?;
        Ok(())
    }
}

#[derive(Debug)]
enum GameNodeState {
    Expanded {is_terminal: bool},
    NotExpanded,
}

type MCTSTree = Tree<GameNode>;

struct MCTS {
    tree: MCTSTree,
    c_puct: f32,
}

impl MCTS {
    pub fn expand(&mut self, leaf_node_id: NodeId) -> f32 {
        let mut leaf_node: NodeMut<'_, GameNode> = self.tree.get_mut(leaf_node_id).unwrap();
        if leaf_node.value().is_terminal() {
            leaf_node.value().node_state = GameNodeState::Expanded { is_terminal: true };

            let status=  leaf_node.value().game_state.status();
            // Because can only win on your own move
            return status.clone().into()
        } else {
            if leaf_node.value().num_children != 0 {panic!("leaf node already has children! (Probably already expanded)")};
            leaf_node.value().node_state = GameNodeState::Expanded { is_terminal: false };

            let valid_moves = leaf_node.value().game_state.valid_moves();

            // Would be calculated from NN
            let prior_probs: Vec<f32> = vec![1. /valid_moves.len() as f32; valid_moves.len()];
            let value: f32 = 0.0;

            leaf_node.value().num_children += valid_moves.len() as u32;

            for (valid_move, prior_prob) in valid_moves.iter().zip(prior_probs.iter()) {
                let mut child_state = leaf_node.value().game_state.clone();
                let child_status = child_state.take_turn(&valid_move);
                // init node, evaluate children an take mean of values as action value
                let child_node = GameNode::new(
                    *prior_prob, 
                    child_state.clone(), 
                    GameNodeState::NotExpanded, 
                    Some(valid_move.clone()));
                
                leaf_node.append(child_node);
            };

            return value
        }
    }
    
    pub fn select(&self) -> Vec<NodeId> {
        // Initialise the search at root
        let mut base_node = self.tree.root();
        let mut node_chain = Vec::<NodeId>::new();
        loop {
            node_chain.push(base_node.id());

            if !matches!(base_node.value().node_state, GameNodeState::Expanded {is_terminal: false}) {
                return node_chain;
            }

            let mut n_vals = Vec::<u32>::new();
            let mut q_vals = Vec::<f32>::new();
            let mut p_vals = Vec::<f32>::new();
            for child_node in base_node.children() {
                let game_node: &GameNode = child_node.value();
                n_vals.push(game_node.num_visits);
                q_vals.push(game_node.action_value);
                p_vals.push(game_node.prior_prob);
            }
            let sum_sqrt: f32 = (n_vals.iter().sum::<u32>() as f32).sqrt();
            let ucts: Vec<f32> = izip!(n_vals, q_vals, p_vals).map(|(n, q, p)| q + self.c_puct * p * sum_sqrt / ((1 + n) as f32)).collect();
            let selected_index = ucts
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.total_cmp(b))
                .map(|(index, _)| index)
                .expect("Children list is empty!");
            base_node = base_node
                .children()
                .nth(selected_index)
                .unwrap();
        }
    }

    pub fn backup(&mut self, node_chain: Vec<NodeId>, value: f32) {
        for node_id in node_chain {
            let mut node = self.tree.get_mut(node_id).expect("No node found");
            let game_node: &mut GameNode = node.value();
            game_node.num_visits += 1;
            game_node.total_value += value;
            game_node.update_action_value();
        }
    }
}

impl Default for MCTS {
    fn default() -> Self {
        Self {
            tree: MCTSTree::new(
                GameNode::new(
                    0.0, 
                    Game::default(),
                    GameNodeState::NotExpanded,
                    None,
                )
            ),
            c_puct: 1.,
        }
    }
}


fn main() {
    let mut mcts = MCTS::default();
    println!("{}", mcts.tree);
    for _ in 0..1000 {
        let node_chain: Vec<NodeId> = mcts.select();
        let value = mcts.expand(node_chain.last().copied().unwrap());
        mcts.backup(node_chain, value);
    }
    // mcts.tree.root().children()
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
