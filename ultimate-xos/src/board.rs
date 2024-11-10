use std::fmt;
use std::ops::{Deref, DerefMut};
use rand::seq::SliceRandom;
use sigmazero::game::{Position, PositionList};

use crate::small_board::Board as SmallBoard;
use crate::small_board::Position3;
pub use crate::small_board::XOPlayer;

#[derive(PartialEq, Clone, Debug)]
pub struct XOPosition {
    x: u8,
    y: u8,
}

impl Position for XOPosition {
    fn new(x: u8, y: u8) -> Self {
        Self {x: x, y: y}
    }

    fn is_valid(&self) -> bool {
        if self.x > 8 || self.y > 8 {
            return false;
        }
        true
    }
}

impl XOPosition {
    fn large_pos(&self) -> Position3 {
        Position3::new(self.x / 3, self.y / 3)
    }

    fn small_pos(&self) -> Position3 {
        Position3::new(self.x % 3, self.y % 3)
    }

    fn from_subpos(large_pos: Position3, small_pos: Position3) -> Self {
        Self {
            x: small_pos.x + 3 * large_pos.x,
            y: small_pos.y + 3 * large_pos.y,
        }
    }
}

impl From<usize> for XOPosition {
    fn from(index: usize) -> Self {
        Self {
            x: (index % 9) as u8,
            y: (index / 9) as u8,
        }
    }
}

impl From<XOPosition> for usize {
    fn from(pos: XOPosition) -> Self {
        (pos.x + 9 * pos.y) as usize
    }
}

impl fmt::Display for XOPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)?;
        Ok(())
    }
}

pub type XOPositionList = PositionList<XOPosition>;

// impl fmt::Display for XOPositionList {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         for y in 0..9 {
//             for x in 0..9 {
//                 let pos = XOPosition::new(x as u8, y as u8);
//                 let cell = self.contains(&pos);

//                 write!(f, " {} ", if cell {"*"} else {" "})?;
//                 if x < 8 {
//                     if x % 3 == 2 {
//                         write!(f, "‖")?;
//                     } else {
//                         write!(f, "|")?;
//                     }
//                 }
//             }
//             if y < 8 {
//                 if y % 3 == 2 {
//                     write!(f, "\n{}", "=".repeat(35))?
//                 } else {
//                     write!(f, "\n{}", "-".repeat(35))?
//                 }
//             }
//             writeln!(f)?;
//         }
//         Ok(())
//     }
// }

#[derive(Clone)]
pub struct MainBoard {
    small_boards: Vec<SmallBoard>,
    board: SmallBoard,
    last_move: Option<XOPosition>,
}

impl MainBoard {
    pub fn get_cell(&self, position: &XOPosition) -> Option<XOPlayer> {
        self.small_boards[position.large_pos().flat() as usize].get_cell(&position.small_pos())
    }

    pub fn set_cell(&mut self, position: &XOPosition, player: XOPlayer) {
        let small_board = &mut self.small_boards[position.large_pos().flat() as usize];
        small_board.set_cell(&position.small_pos(), player);
        match small_board.winner() {
            Some(winner) => self.board.set_cell(&position.large_pos(), winner),
            None => (),
        };
        self.last_move = Some(position.clone());
    }

    pub fn winner(&self) -> Option<XOPlayer> {
        self.board.winner()
    }

    pub fn is_valid_move(&self, position: &XOPosition) -> bool {
        if !position.is_valid() {
            return false
        }
        // If not first move
        match &self.last_move {
            Some(last_move) => {
                let target_small_board= &self.small_boards[last_move.small_pos().flat() as usize];
                if position.large_pos() != last_move.small_pos() {
                    if !(target_small_board.valid_moves().len() == 0) {
                        return false;
                    };
                } else {
                    return target_small_board
                                .get_cell(&position.small_pos())
                                .is_none()
                }
            },
            None => (),
        }
        // If first move or target small board is full
        self.get_cell(&position).is_none()
    }

    fn available_cells(&self) -> XOPositionList {
        let mut available_cells = Vec::new();
        for i in 0..9 {
            let small_board_valid_moves = self.small_boards[i].valid_moves();
            let large_pos = Position3::from_flat(i as u8);
            for small_pos in small_board_valid_moves {
                available_cells.push(XOPosition::from_subpos(large_pos.clone(), small_pos))
            }
        }
        XOPositionList::new(available_cells)
    }

    pub fn valid_moves(&self) -> XOPositionList {
        match &self.last_move {
            None => {return self.available_cells();},
            Some(last_move) => {
                let target_small_board= &self.small_boards[last_move.small_pos().flat() as usize];
                if target_small_board.valid_moves().len() == 0 {
                    return self.available_cells();
                } else {
                    let mut cells = Vec::new();
                    for p_small in target_small_board.valid_moves() {
                        cells.push(XOPosition::from_subpos(last_move.small_pos(), p_small))
                    }
                    XOPositionList::new(cells)
                }
            },
        }
    }

    pub fn is_draw(&self) -> bool {
        self.available_cells().len() == 0
    }
}

impl fmt::Display for MainBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..9 {
            for x in 0..9 {
                let pos = XOPosition::new(x as u8, y as u8);
                let cell = self.get_cell(&pos);

                let last_move_mark = match self.last_move.clone() {
                    Some(last_move) => {if last_move == pos {"-"} else {" "}},
                    None => " ",
                };
                write!(f, "{last_move_mark}{}{last_move_mark}", cell.map_or(" ".to_string(), |p| p.to_string()))?;
                if x < 8 {
                    if x % 3 == 2 {
                        write!(f, "‖")?;
                    } else {
                        write!(f, "|")?;
                    }
                }
            }
            if y < 8 {
                if y % 3 == 2 {
                    write!(f, "\n{}", "=".repeat(35))?
                } else {
                    write!(f, "\n{}", "-".repeat(35))?
                }
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

pub fn play_random_game() -> Option<XOPlayer> {
    let mut board = MainBoard::default();
    let mut rng = rand::thread_rng();
    let mut player = XOPlayer::X;

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
