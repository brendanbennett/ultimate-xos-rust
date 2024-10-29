#![feature(test)]
extern crate test;

mod small_board;

use std::fmt;
use rand::seq::SliceRandom;

use small_board::Board as SmallBoard;
use small_board::{Player, Position3};

#[derive(PartialEq, Clone)]
pub struct Position {
    x: u8,
    y: u8,
}

impl Position {
    pub fn new(x: u8, y: u8) -> Self {
        Self {x: x, y: y}
    }

    pub fn from_vec(vec: Vec<u32>) -> Result<Self, String> {
        if vec.len() != 2 {
            return Err("Incorrect number of coordinates".to_string())
        }
        Ok(Self {x: vec[0] as u8, y: vec[1] as u8})
    }

    pub fn is_valid(&self) -> bool {
        if self.x > 8 || self.y > 8 {
            return false;
        }
        true
    }

    pub fn large_pos(&self) -> Position3 {
        Position3::new(self.x / 3, self.y / 3)
    }

    pub fn small_pos(&self) -> Position3 {
        Position3::new(self.x % 3, self.y % 3)
    }

    fn from_subpos(large_pos: Position3, small_pos: Position3) -> Self {
        Self {
            x: small_pos.x + 3 * large_pos.x,
            y: small_pos.y + 3 * large_pos.y,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[{}, {}]", self.x, self.y)?;
        Ok(())
    }
}

pub struct MainBoard {
    small_boards: Vec<SmallBoard>,
    board: SmallBoard,
    last_move: Option<Position>,
}

impl MainBoard {
    pub fn get_cell(&self, position: &Position) -> Option<Player> {
        self.small_boards[position.large_pos().flat() as usize].get_cell(&position.small_pos())
    }

    pub fn set_cell(&mut self, position: &Position, player: Player) {
        let small_board = &mut self.small_boards[position.large_pos().flat() as usize];
        small_board.set_cell(&position.small_pos(), player);
        match small_board.winner() {
            Some(winner) => self.board.set_cell(&position.large_pos(), winner),
            None => (),
        };
        self.last_move = Some(position.clone());
    }

    pub fn winner(&self) -> Option<Player> {
        self.board.winner()
    }

    pub fn is_valid_move(&self, position: &Position) -> bool {
        // If not first move
        let target_small_board= &self.small_boards[position.large_pos().flat() as usize];
        match &self.last_move {
            Some(last_move) => {
                if position.large_pos() != last_move.small_pos()
                    && !target_small_board.is_full() { return false }
            },
            None => (),
        }
        // Check for empty
        target_small_board
            .get_cell(&position.small_pos())
            .is_none()
    }

    fn empty_cells(&self) -> Vec<Position> {
        let mut empty_cells = Vec::new();
        for x in 0..9 {
            for y in 0..9 {
                let pos = Position::new(x, y);
                if self.get_cell(&pos).is_none() {
                    empty_cells.push(pos)
                }
            }
        }
        empty_cells
    }

    pub fn valid_moves(&self) -> Vec<Position> {
        match &self.last_move {
            None => {return self.empty_cells();},
            Some(last_move) => {
                let target_small_board= &self.small_boards[last_move.small_pos().flat() as usize];
                if target_small_board.is_full() {
                    return self.empty_cells();
                } else {
                    let mut cells = Vec::new();
                    for p_small in target_small_board.empty_cells() {
                        cells.push(Position::from_subpos(last_move.small_pos(), p_small))
                    }
                    cells
                }
            },
        }

    }
}

impl fmt::Display for MainBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..9 {
            for x in 0..9 {
                let cell = self.get_cell(&Position::new(x as u8, y as u8));
                write!(f, " {} ", cell.map_or(" ".to_string(), |p| p.to_string()))?;
                if x < 8 {
                    write!(f, "|")?;
                }
            }
            if y < 8 {
                write!(f, "\n{}", "-".repeat(35))?
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Default for MainBoard {
    fn default() -> Self {
        Self {
            small_boards: vec![SmallBoard::default(); 9],
            board: SmallBoard::default(),
            last_move: None,
        }
    }
}

pub fn play_random_game() -> Option<Player> {
    let mut board = MainBoard::default();
    let mut rng = rand::thread_rng();
    let mut player = Player::X;

    loop {
        match board.valid_moves().choose(&mut rng) {
            Some(mv) => board.set_cell(mv, player),
            None => break None,
        }
        // println!("{board}");
        match board.winner() {
            Some(winner) => {println!("Player {winner} wins!"); break Some(winner);},
            None => (),
        }
        player = player.other_player();
    }
}

fn main() {
    let mut board = MainBoard::default();
    let mut rng = rand::thread_rng();
    let mut player = Player::X;

    loop {
        match board.valid_moves().choose(&mut rng) {
            Some(mv) => board.set_cell(mv, player),
            None => break,
        }
        // println!("{board}");
        match board.winner() {
            Some(winner) => {println!("Player {winner} wins!"); break;},
            None => (),
        }
        player = player.other_player();
    }
}

#[cfg(test)]
mod benchmarks {
    use test::Bencher;

    use super::play_random_game;

    #[bench]
    fn bench_play_game(b: &mut Bencher) {
        b.iter(|| play_random_game());
    }
}
