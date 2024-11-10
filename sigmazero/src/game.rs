use std::fmt;
use std::ops::{Deref, DerefMut};

pub trait Position: PartialEq + Clone + fmt::Debug + fmt::Display + From<usize> + Into<usize> {
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

#[derive(Debug, Clone)]
pub enum GameStatus<P: Player> {
    InProgress {player: P},
    Won {player: P},
    Draw,
}

impl<P: Player> Into<f32> for GameStatus<P> {
    fn into(self) -> f32 {
        match self {
            Self::InProgress { player: _ } => 0.0,
            Self::Draw => 0.0,
            Self::Won { player: _ } => 1.0,
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

pub trait Game: Default + Clone {
    const N: usize;

    type Player: Player;
    type Position: Position;

    fn take_turn(&mut self, position: &Self::Position) -> Result<GameStatus<Self::Player>, GameError<Self::Position>>;
    fn valid_moves(&self) -> PositionList<Self::Position>;
    fn status(&self) -> &GameStatus<Self::Player>;
}
