mod small_board;

use std::fmt;

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
}

// TODO: Refactor to be method in Position
pub struct SubPositions {
    large_pos: Position3,
    small_pos: Position3,
}

impl SubPositions {
    pub fn from_position(position: &Position) -> Self {
        Self {
            large_pos: Position3::new(position.x / 3, position.y / 3),
            small_pos: Position3::new(position.x % 3, position.y % 3),
        }
    }
}

pub struct MainBoard {
    small_boards: Vec<SmallBoard>,
    board: SmallBoard,
    last_move: Option<Position>,
}

impl MainBoard {
    pub fn get_cell(&self, position: &Position) -> Option<Player> {
        let sub_pos = SubPositions::from_position(position);
        self.small_boards[sub_pos.large_pos.flat() as usize].get_cell(&sub_pos.small_pos)
    }

    pub fn set_cell(&mut self, position: &Position, player: Player) {
        let sub_pos = SubPositions::from_position(position);
        let small_board = &mut self.small_boards[sub_pos.large_pos.flat() as usize];
        small_board.set_cell(&sub_pos.small_pos, player);
        match small_board.winner() {
            Some(winner) => self.board.set_cell(&sub_pos.large_pos, winner),
            None => (),
        };
        self.last_move = Some(position.clone());
    }

    fn small_board_from_position(&self, position: &Position) -> &SmallBoard {
        let sub_pos = SubPositions::from_position(position);
        &self.small_boards[sub_pos.large_pos.flat() as usize]
    }

    pub fn winner(&self) -> Option<Player> {
        self.board.winner()
    }

    pub fn is_valid_move(&self, position: &Position) -> bool {
        let sub_pos = SubPositions::from_position(position);
        // If not first move
        match &self.last_move {
            Some(last_move) => {
                if sub_pos.large_pos != SubPositions::from_position(last_move).small_pos
                    && !self.small_boards[sub_pos.large_pos.flat() as usize].is_full() { return false }
            },
            None => (),
        }
        // Check for empty
        self.small_boards[sub_pos.large_pos.flat() as usize]
            .get_cell(&sub_pos.small_pos)
            .is_none()
    }

    pub fn valid_moves(&self) -> Vec<Position> {
        match &self.last_move {
            Some(last_move) => {
                if !self.small_boards[last_move.flat() as usize].is_full() { return false }
            },
            None => (),
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


fn main() {
    let mut board = MainBoard::default();
    println!("{}", board);
    board.set_cell(&Position::new(5, 2), Player::X);
    println!("{}", board);
    board.set_cell(&Position::new(5, 2), Player::X);
}
