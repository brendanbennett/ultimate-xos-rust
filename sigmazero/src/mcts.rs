use core::fmt;

use crate::data::ReplayBuffer;
use crate::game::{Game, GameStatus};
use crate::policy::{Agent, RawPolicy};
use ego_tree::{NodeId, NodeMut, NodeRef, Tree};
use indicatif::ProgressIterator;
use itertools::izip;

pub struct GameNode<G: Game<N>, const N: usize> {
    num_visits: u32,
    prior_prob: f32,
    total_value: f32,
    action_value: f32,
    pub game_state: G,
    node_state: GameNodeState,
    previous_action: Option<G::Position>, // Only None for root
}

impl<G: Game<N>, const N: usize> GameNode<G, N> {
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

impl<G: Game<N>, const N: usize> fmt::Display for GameNode<G, N> {
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
pub enum GameNodeState {
    Expanded { is_terminal: bool },
    NotExpanded,
}

type MCTSTree<G, const N: usize> = Tree<GameNode<G, N>>;

pub struct MCTS<'a, G: Game<N>, A: Agent<G, N>, const N: usize> {
    tree: MCTSTree<G, N>,
    c_puct: f32,
    agent: &'a mut A,
}

impl<'a, G: Game<N>, A: Agent<G, N>, const N: usize> MCTS<'a, G, A, N> {
    pub fn expand(&mut self, leaf_node_id: NodeId) -> f32 {
        let mut leaf_node: NodeMut<'_, GameNode<G, N>> = self.tree.get_mut(leaf_node_id).unwrap();
        if leaf_node.value().is_terminal() {
            leaf_node.value().node_state = GameNodeState::Expanded { is_terminal: true };

            let status = leaf_node.value().game_state.status();
            // Because can only win on your own move
            return status.into();
        } else {
            if leaf_node.has_children() {
                panic!("leaf node already has children! (Probably already expanded)")
            };
            leaf_node.value().node_state = GameNodeState::Expanded { is_terminal: false };

            // Would be calculated from NN
            let (policy, value) = self.agent.eval_game(&leaf_node.value().game_state);

            for (valid_move, prior_prob) in policy.mask_policy(&leaf_node.value().game_state) {
                let mut child_state = leaf_node.value().game_state;
                _ = child_state.take_turn(&valid_move);
                // init node, evaluate children an take mean of values as action value
                let child_node = GameNode::new(
                    prior_prob,
                    child_state,
                    GameNodeState::NotExpanded,
                    Some(valid_move),
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
                let game_node: &GameNode<G, N> = child_node.value();
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
            let game_node: &mut GameNode<G, N> = node.value();
            game_node.num_visits += 1;
            game_node.total_value += value;
            game_node.update_action_value();

            value = value * -1.;
        }
    }

    pub fn select_best_child(&self) -> (&GameNode<G, N>, RawPolicy<N>) {
        let mut num_sum: f32 = 0.0;
        let mut policy: [f32; N] = [0.0; N];
        let mut max_num: u32 = 0;
        let mut best_child: Option<NodeRef<'_, GameNode<G, N>>> = None;
        for child in self.tree.root().children() {
            num_sum += child.value().num_visits as f32;
            if child.value().num_visits > max_num {
                // always takes first best value
                max_num = child.value().num_visits;
                best_child = Some(child);
            }
            policy[child.value().previous_action.unwrap().into()] = child.value().num_visits as f32;
        }
        policy = policy.map(|n| n / num_sum);

        (
            best_child.expect("No children found!").value(),
            RawPolicy::new(policy),
        )
    }

    pub fn from_root_game_state(root_game_state: G, agent: &'a mut A) -> Self {
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

pub fn self_play<G: Game<N>, A: Agent<G, N>, const N: usize>(
    agent: &mut A,
    n_games: usize,
    verbose: bool,
) -> ReplayBuffer<G, N> {
    let mut buffer = ReplayBuffer::default();

    for _ in (0..n_games).progress() {
        let mut games = vec![G::default()];
        let mut values = Vec::<f32>::new();
        let mut policies = Vec::<RawPolicy<N>>::new();
        loop {
            let mut mcts =
                MCTS::<G, A, N>::from_root_game_state(games.last().unwrap().clone(), agent);
            // println!("{}", mcts.tree);
            for _ in 0..800 {
                let node_chain: Vec<NodeId> = mcts.select();
                let value = mcts.expand(node_chain.last().copied().unwrap());
                mcts.backup(node_chain, value);
            }

            // for child in mcts.tree.root().children() {
            //     println!("{} has N={}", child.value().previous_action.clone().map_or("Root".to_string(), |p| p.to_string()), child.value().num_visits)
            // }

            let (best_child, raw_policy) = mcts.select_best_child();

            if verbose {
                print!("{esc}c", esc = 27 as char);
                println!("{}", best_child.game_state);
            }

            policies.push(raw_policy);

            if best_child.is_terminal() {
                if verbose {
                    println!("Result: {:?}", best_child.game_state.status());
                }
                let final_value: f32 = best_child.game_state.status().into();
                for i in 0..games.len() {
                    if i % 2 == 0 {
                        values.push(final_value);
                    } else {
                        values.push(-final_value);
                    }
                }
                values.reverse();
                break;
            }
            games.push(best_child.game_state);
        }
        buffer.append(&mut games, &mut values, &mut policies);
    }
    buffer
}
