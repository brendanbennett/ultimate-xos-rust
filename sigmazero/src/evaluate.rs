use std::ops::Index;

use crate::{game::{Game, GameStatus, Player}, mcts::MCTS, policy::Agent};
use indicatif::ProgressIterator;
use ego_tree::NodeId;

#[derive(Debug, Default)]
pub struct EvaluationResults {
    pub agent1_wins: usize,
    pub agent2_wins: usize,
    pub draws: usize,
}

pub fn evaluate_agents<G: Game<N>, const N: usize, A1: Agent<G, N>, A2: Agent<G, N>>(agent1: &mut A1, agent2: &mut A2, n_games: usize, search_steps: usize, verbose: bool) -> EvaluationResults {
    let mut results = EvaluationResults::default();

    for _ in (0..n_games).progress() {
        let mut game = G::default();

        loop {
            match game.status() {
                GameStatus::InProgress { player } => {
                    if player == &G::Player::PLAYERS[0] {
                        // Agent 1's turn
                        let mut mcts1 = MCTS::<G, A1, N>::from_root_game_state(game.clone(), agent1);
                        
                        // Perform search steps
                        for _ in 0..search_steps {
                            let node_chain = mcts1.select();
                            if let Some(last_node) = node_chain.last() {
                                let value = mcts1.expand(*last_node);
                                mcts1.backup(node_chain, value);
                            }
                        }
                        
                        
                        game = mcts1.select_best_child().0.game_state.clone();
                    } else {
                        // Agent 2's turn
                        let mut mcts2 = MCTS::<G, A2, N>::from_root_game_state(game.clone(), agent2);
                        
                        // Perform search steps
                        for _ in 0..search_steps {
                            let node_chain = mcts2.select();
                            if let Some(last_node) = node_chain.last() {
                                let value = mcts2.expand(*last_node);
                                mcts2.backup(node_chain, value);
                            }
                        }
                        
                        game = mcts2.select_best_child().0.game_state.clone();
                    };

                    if verbose {
                        print!("{esc}c", esc = 27 as char);
                        println!("{}", game);
                    }

                },
                GameStatus::Draw => {
                    results.draws += 1;
                    if verbose {
                        println!("Result: Draw");
                    }
                    break;
                }
                GameStatus::Won { player } => {
                    if player == &G::Player::PLAYERS[0] {
                        results.agent1_wins += 1;
                    } else {
                        results.agent2_wins += 1;
                    }
                    if verbose {
                        println!("Result: Player {:?} won", player);
                    }
                    break;
                }
            };
        }
    }
    results
}