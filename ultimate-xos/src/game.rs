use core::panic;
use std::fmt;

pub use crate::board::XOPlayer;
use crate::board::{BoardDisplayer, MainBoard, XOPosition, XOPositionList};
use sigmazero::{game::{Game, GameError, GameStatus}, policy::RawPolicy};

pub type XOGameStatus = GameStatus<XOPlayer>;

#[derive(Clone, Copy)]
pub struct XOGame {
    board: MainBoard,
    status: GameStatus<XOPlayer>,
}

impl Default for XOGame {
    fn default() -> Self {
        Self {
            board: MainBoard::default(),
            status: GameStatus::default(),
        }
    }
}

impl Game<81> for XOGame {
    const FEATURES_SHAPE: &'static [i64] = &[3, 9, 9];
    const FEATURES_SIZE: i64 = 3 * 9 * 9;

    type Player = XOPlayer;
    type Position = XOPosition;

    fn take_turn(
        &mut self,
        position: &Self::Position,
    ) -> Result<XOGameStatus, GameError<Self::Position>> {
        let current_player = match self.status {
            GameStatus::InProgress { player } => player,
            _ => return Err(GameError::GameOver),
        };

        if !self.board.is_valid_move(position) {
            return Err(GameError::InvalidMove {
                position: *position,
            });
        }

        self.board.set_cell(position, current_player);

        self.status = if let Some(winner) = self.board.winner() {
            GameStatus::Won { player: winner }
        } else if self.board.is_draw() {
            GameStatus::Draw
        } else {
            GameStatus::InProgress {
                player: current_player.other_player(),
            }
        };

        Ok(self.status)
    }

    fn valid_moves(&self) -> XOPositionList {
        self.board.valid_moves()
    }

    fn status(&self) -> &GameStatus<XOPlayer> {
        &self.status
    }

    fn displays(items: Vec<String>) -> impl fmt::Display {
        BoardDisplayer::new(items)
    }

    fn features(&self) -> tch::Tensor {
        let current_player = match self.status {
            GameStatus::InProgress { player } => player,
            _ => panic!("Features for completed game? In esta economia?"),
        };
        tch::Tensor::from_slice(
            self.board
                .features_for_player(current_player)
                .iter()
                .flatten()
                .flatten()
                .copied()
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .reshape([3, 9, 9])
        .to_dtype(tch::Kind::Float, false, false)
    }

    fn augmented_with_raw_policy(&self, raw_policy: &RawPolicy<81>) -> (Vec<Self>, Vec<RawPolicy<81>>) {
        let aug_boards = self.board.augmented();
        let mut aug_games = vec![self.clone()];
        let aug_policies = Self::augment_raw_policy(raw_policy);
        for r in 1..4 {
            aug_games.push(Self { board: aug_boards[r], status: self.status.clone() })
        }

        (aug_games, aug_policies)
    }
}

impl XOGame {
    fn augment_raw_policy(raw_policy: &RawPolicy<81>) -> Vec<RawPolicy<81>>{
        let mut aug_policies = vec![raw_policy.clone()];
        for r in 1..4 {
            aug_policies.push(Self::rotate_raw_policy_90(&aug_policies[r-1]));
        }
        aug_policies
    }

    fn rotate_raw_policy_90(raw_policy: &RawPolicy<81>) -> RawPolicy<81> {
        let mut rot_policy = [0.0f32; 81];
        for x in 0..9 {
            for y in 0..9 {
                rot_policy[x + 9 * y] = raw_policy[y + 9 * (8-x)]
            }
        }
        RawPolicy::new(rot_policy)
    }
}

impl fmt::Display for XOGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.board)
    }
}

#[cfg(test)]
mod tests {
    use crate::policies;

    use super::*;

    #[test]
    fn test_rotation() {
        // Create a test grid where each cell contains its index (0 to 80)
        let mut initial = [0.0f32; 81];
        for i in 0..81 {
            initial[i] = i as f32;
        }
        let policy = RawPolicy::new(initial);

        println!("Original grid:");

        let rotated = XOGame::rotate_raw_policy_90(&policy);
        println!("Rotated grid:");

        // Test specific positions
        // After 90-degree clockwise rotation:
        // - (0,0) -> (8,0) : 0 should move to index 8
        // - (8,0) -> (8,8) : 8 should move to index 80
        // - (0,8) -> (0,0) : 72 should move to index 0
        // - (8,8) -> (0,8) : 80 should move to index 72
        assert_eq!(rotated[8], 0.0);    // top-left -> top-right
        assert_eq!(rotated[80], 8.0);   // top-right -> bottom-right
        assert_eq!(rotated[0], 72.0);   // bottom-left -> top-left
        assert_eq!(rotated[72], 80.0);  // bottom-right -> bottom-left

        // Test middle cell - should stay the same value
        assert_eq!(rotated[40], 40.0);  // center should be unchanged

        // Test a few more positions
        assert_eq!(rotated[17], 1.0);   // (1,0) -> (8,1)
        assert_eq!(rotated[7], 9.0);    // (0,1) -> (7,0)
        assert_eq!(rotated[63], 79.0);  // (7,8) -> (0,7)
        assert_eq!(rotated[10], 64.0);  // (1,7) -> (1,1)

        assert_eq!(policy, XOGame::rotate_raw_policy_90(
            &XOGame::rotate_raw_policy_90(
                &XOGame::rotate_raw_policy_90(
                    &XOGame::rotate_raw_policy_90(&policy)
                )
            )
        ));
    }
}