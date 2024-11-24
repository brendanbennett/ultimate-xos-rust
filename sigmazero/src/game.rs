use std::fmt::{self, Debug, Display};
use std::ops::{Deref, DerefMut};

pub trait Position: PartialEq + Clone + Copy + fmt::Debug + fmt::Display + From<usize> + Into<usize> {
    fn new(x: u8, y: u8) -> Self;
    fn is_valid(&self) -> bool;
}

pub struct PositionList<P: Position>(Vec<P>);

impl<P: Position> PositionList<P> {
    pub fn new(positions: Vec<P>) -> Self {
        Self(positions)
    }
}

impl<P: Position> Deref for PositionList<P> {
    type Target = Vec<P>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<P: Position> DerefMut for PositionList<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait Player: fmt::Debug + Clone + Copy + PartialEq + Default + fmt::Display {
    const PLAYERS: [Self; 2];

    fn other_player(&self) -> Self;
}

#[derive(Debug, Clone, Copy)]
pub enum GameStatus<P: Player> {
    InProgress {player: P},
    Won {player: P},
    Draw,
}

impl<P: Player> From<GameStatus<P>> for f32 {
    fn from(value: GameStatus<P>) -> Self {
        match value {
            GameStatus::InProgress { player: _ } => 0.0,
            GameStatus::Draw => 0.0,
            GameStatus::Won { player: _ } => 1.0,
        }
    }
}

impl<P: Player> From<&GameStatus<P>> for f32 {
    fn from(value: &GameStatus<P>) -> Self {
        match value {
            GameStatus::InProgress { player: _ } => 0.0,
            GameStatus::Draw => 0.0,
            GameStatus::Won { player: _ } => 1.0,
        }
    }
}

impl<P: Player> Default for GameStatus<P> {
    fn default() -> Self {
        Self::InProgress { player: P::default() }
    }
}

#[derive(Debug)]
pub enum GameError<P: Position> {
    InvalidMove { position: P },
    GameOver,
}

pub trait Game<const N: usize>: Default + Clone + Copy + Display + Debug {
    const MAX_ACTIONS: usize = N;
    const FEATURES_SHAPE: &'static [i64];
    const FEATURES_SIZE: i64;


    type Player: Player;
    type Position: Position;

    fn take_turn(&mut self, position: &Self::Position) -> Result<GameStatus<Self::Player>, GameError<Self::Position>>;
    fn valid_moves(&self) -> PositionList<Self::Position>;
    fn status(&self) -> &GameStatus<Self::Player>;
    fn displays(items: Vec<String>) -> impl Display;
    fn features(&self) -> tch::Tensor;
}
