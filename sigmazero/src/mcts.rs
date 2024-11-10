
use core::fmt;

use ego_tree::{NodeId, NodeMut, NodeRef, Tree};
use crate::game::{Game, GameStatus, Position};
use crate::policy::{Agent};
use itertools::izip;
use rand::prelude::*;

pub struct GameNode<G: Game> {
    num_visits: u32,
    prior_prob: f32,
    total_value: f32,
    action_value: f32,
    pub game_state: G,
    node_state: GameNodeState,
    previous_action: Option<G::Position>, // Only None for root
}

impl<G: Game> GameNode<G> {
    pub fn new(
        prior_prob: f32,
        game_state: G,
        node_state: GameNodeState,
        previous_action: Option<G::Position>,
    ) -> Self {
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

    pub fn is_terminal(&self) -> bool {
        !matches!(
            self.game_state.status(),
            GameStatus::InProgress { player: _ }
        )
    }

    pub fn update_action_value(&mut self) {
        self.action_value = self.total_value / self.num_visits as f32;
    }
}

impl<G: Game> fmt::Display for GameNode<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, ",
            if let Some(prev_action) = &self.previous_action {
                prev_action.to_string()
            } else {
                "Root".to_string()
            }
        )?;
        write!(f, "{:?}, N={}", self.node_state, self.num_visits)?;
        Ok(())
    }
}

#[derive(Debug)]
enum GameNodeState {
    Expanded { is_terminal: bool },
    NotExpanded,
}

type MCTSTree<G: Game> = Tree<GameNode<G>>;

pub struct MCTS<G: Game, A: Agent<G>> {
    tree: MCTSTree<G>,
    c_puct: f32,
    agent: A,
}

impl<G: Game, A: Agent<G>> MCTS<G, A> {
    pub fn expand(&mut self, leaf_node_id: NodeId) -> f32 {
        let mut leaf_node: NodeMut<'_, GameNode<G>> = self.tree.get_mut(leaf_node_id).unwrap();
        if leaf_node.value().is_terminal() {
            leaf_node.value().node_state = GameNodeState::Expanded { is_terminal: true };

            let status = leaf_node.value().game_state.status();
            // Because can only win on your own move
            return status.clone().into();
        } else {
            if leaf_node.has_children() {
                panic!("leaf node already has children! (Probably already expanded)")
            };
            leaf_node.value().node_state = GameNodeState::Expanded { is_terminal: false };

            // Would be calculated from NN
            let (policy, value)= self.agent.eval(&leaf_node.value().game_state);

            for (valid_move, prior_prob) in policy {
                let mut child_state = leaf_node.value().game_state.clone();
                _ = child_state.take_turn(&valid_move);
                // init node, evaluate children an take mean of values as action value
                let child_node = GameNode::new(
                    prior_prob,
                    child_state.clone(),
                    GameNodeState::NotExpanded,
                    Some(valid_move.clone()),
                );

                leaf_node.append(child_node);
            }

            return value;
        }
    }

    pub fn select(&self) -> Vec<NodeId> {
        // Initialise the search at root
        let mut base_node = self.tree.root();
        let mut node_chain = Vec::<NodeId>::new();
        loop {
            node_chain.push(base_node.id());

            if !matches!(
                base_node.value().node_state,
                GameNodeState::Expanded { is_terminal: false }
            ) {
                return node_chain;
            }

            let mut n_vals = Vec::<u32>::new();
            let mut q_vals = Vec::<f32>::new();
            let mut p_vals = Vec::<f32>::new();
            for child_node in base_node.children() {
                let game_node: &GameNode<G> = child_node.value();
                n_vals.push(game_node.num_visits);
                q_vals.push(game_node.action_value);
                p_vals.push(game_node.prior_prob);
            }
            let sum_sqrt: f32 = (n_vals.iter().sum::<u32>() as f32).sqrt();
            let ucts: Vec<f32> = izip!(n_vals, q_vals, p_vals)
                .map(|(n, q, p)| q + self.c_puct * p * sum_sqrt / ((1 + n) as f32))
                .collect();
            let selected_index = ucts
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.total_cmp(b))
                .map(|(index, _)| index)
                .expect("Children list is empty!");
            base_node = base_node.children().nth(selected_index).unwrap();
        }
    }

    pub fn backup(&mut self, node_chain: Vec<NodeId>, value: f32) {
        let mut value = value;
        for node_id in node_chain.into_iter().rev() {
            let mut node = self.tree.get_mut(node_id).expect("No node found");
            let game_node: &mut GameNode<G> = node.value();
            game_node.num_visits += 1;
            game_node.total_value += value;
            game_node.update_action_value();

            value = value * -1.;
        }
    }

    pub fn select_best_child(&self) -> &GameNode<G> {
        let node_id = self.tree
            .root()
            .children()
            .max_by(|x, y| x.value().num_visits.cmp(&y.value().num_visits))
            .unwrap()
            .id();
        self.tree.get(node_id).unwrap().value()
    }

    pub fn from_root_game_state(root_game_state: G, agent: A) -> Self {
        Self {
            tree: MCTSTree::new(GameNode::new(
                0.0,
                root_game_state,
                GameNodeState::NotExpanded,
                None,
            )),
            c_puct: 1.,
            agent,
        }
    }
}


impl<G: Game, A: Agent<G>> Default for MCTS<G, A> {
    fn default() -> Self {
        Self {
            tree: MCTSTree::new(GameNode::new(
                0.0,
                G::default(),
                GameNodeState::NotExpanded,
                None,
            )),
            c_puct: 1.,
            agent: A::new(),
        }
    }
}
