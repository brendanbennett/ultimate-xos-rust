#![feature(test)]
extern crate test;

mod board;
mod small_board;
mod game;

use core::fmt;

use board::{Position, Player};

use rand::prelude::*;
use itertools::izip;
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
            if leaf_node.has_children() {panic!("leaf node already has children! (Probably already expanded)")};
            leaf_node.value().node_state = GameNodeState::Expanded { is_terminal: false };

            let valid_moves = leaf_node.value().game_state.valid_moves();

            // Would be calculated from NN
            let prior_probs: Vec<f32> = vec![1. /valid_moves.len() as f32; valid_moves.len()];
            let value: f32 = (random::<f32>()-0.5)*0.2;

            for (valid_move, prior_prob) in valid_moves.iter().zip(prior_probs.iter()) {
                let mut child_state = leaf_node.value().game_state.clone();
                _ = child_state.take_turn(&valid_move);
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
        let mut value = value;
        for node_id in node_chain.into_iter().rev() {
            let mut node = self.tree.get_mut(node_id).expect("No node found");
            let game_node: &mut GameNode = node.value();
            game_node.num_visits += 1;
            game_node.total_value += value;
            game_node.update_action_value();
            
            value = value * -1.;
        }
    }

    pub fn select_best_child(&self) -> NodeId {
        self.tree
            .root()
            .children()
            .max_by(|x, y| x.value().num_visits.cmp(&y.value().num_visits))
            .unwrap()
            .id()
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

impl MCTS {
    fn from_root_game_state(root_game_state: Game) -> Self {
        Self {
            tree: MCTSTree::new(
                GameNode::new(
                    0.0, 
                    root_game_state,
                    GameNodeState::NotExpanded,
                    None,
                )
            ),
            c_puct: 1.,
        }
    }
}


fn main() {
    let mut rng = rand::thread_rng();
    let mut root_game = Game::default();
    loop {
        if matches!(root_game.status(), GameStatus::InProgress{player: Player::O}) {
            _ = root_game.take_turn(root_game.valid_moves().choose(&mut rng).unwrap());
        }
        let mut mcts = MCTS::from_root_game_state(root_game.clone());
        // println!("{}", mcts.tree);
        for _ in 0..1000 {
            let node_chain: Vec<NodeId> = mcts.select();
            let value = mcts.expand(node_chain.last().copied().unwrap());
            mcts.backup(node_chain, value);
        }
        
        for child in mcts.tree.root().children() {
            // println!("{} has N={}", child.value().previous_action.clone().map_or("Root".to_string(), |p| p.to_string()), child.value().num_visits)
        }

        let best_child_id = mcts.select_best_child();
        let best_child: &GameNode = mcts.tree.get(best_child_id).unwrap().value();
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
