use colored::Colorize;
use rand::seq::SliceRandom;
use sigmazero::game::{Position, PositionList};
use std::fmt;

use crate::small_board::Board as SmallBoard;
use crate::small_board::Position3;
pub use crate::small_board::XOPlayer;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct XOPosition {
    x: u8,
    y: u8,
}

impl Position for XOPosition {
    fn new(x: u8, y: u8) -> Self {
        Self { x: x, y: y }
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
    
    fn rot_90(&self) -> Self {
        Self::new(8-self.y, self.x)
    }
    
    /// Reflects the x coordinate, y coordinate is unchanged
    fn reflect_vertical(&self) -> Self {
        Self::new(8-self.x, self.y)
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

#[derive(Clone, Copy)]
pub struct MainBoard {
    small_boards: [SmallBoard; 9],
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
        self.last_move = Some(*position);
    }

    pub fn winner(&self) -> Option<XOPlayer> {
        self.board.winner()
    }

    pub fn is_valid_move(&self, position: &XOPosition) -> bool {
        if !position.is_valid() {
            return false;
        }
        // If not first move
        match &self.last_move {
            Some(last_move) => {
                let target_small_board = &self.small_boards[last_move.small_pos().flat() as usize];
                if position.large_pos() != last_move.small_pos() {
                    if !(target_small_board.valid_moves().len() == 0) {
                        return false;
                    };
                } else {
                    return target_small_board.get_cell(&position.small_pos()).is_none();
                }
            }
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
            None => {
                return self.available_cells();
            }
            Some(last_move) => {
                let target_small_board = &self.small_boards[last_move.small_pos().flat() as usize];
                if target_small_board.valid_moves().len() == 0 {
                    return self.available_cells();
                } else {
                    let mut cells = Vec::new();
                    for p_small in target_small_board.valid_moves() {
                        cells.push(XOPosition::from_subpos(last_move.small_pos(), p_small))
                    }
                    XOPositionList::new(cells)
                }
            }
        }
    }

    pub fn is_draw(&self) -> bool {
        // FIXME If the only move available leads to draw (i.e. fills in the target small board) then this doesn't seem to work??
        self.available_cells().len() == 0
    }

    pub fn features_for_player(&self, player: XOPlayer) -> [[[i64; 9]; 9]; 3] {
        // [current player, other player, last move]
        let mut arr: [[[i64; 9]; 9]; 3] = [[[0; 9]; 9]; 3];
        for y in 0..9 {
            for x in 0..9 {
                let cell = self.get_cell(&XOPosition::new(x, y));
                match cell {
                    Some(p) => {
                        if p == player {
                            arr[0][y as usize][x as usize] = 1
                        } else {
                            arr[1][y as usize][x as usize] = 1
                        }
                    }
                    None => (),
                }
            }
        }
        if let Some(last_move) = self.last_move {
            arr[2][last_move.y as usize][last_move.x as usize] = 1
        }
        arr
    }

    fn rotated_90(&self) -> Self {
        let mut rotated = Self::default();
        // Temporary
        for x in 0..9 {
            for y in 0..9 {
                let pos = XOPosition::new(x, y);
                if let Some(player) = self.get_cell(&pos) {
                    rotated.set_cell(&pos.rot_90(), player);
                }
            }
        }
        rotated.last_move = self.last_move.map(|p| p.rot_90());
        rotated
    }

    fn reflected_vertical(&self) -> Self {
        let mut reflected = Self::default();
        // Temporary
        for x in 0..9 {
            for y in 0..9 {
                let pos = XOPosition::new(x, y);
                if let Some(player) = self.get_cell(&pos) {
                    reflected.set_cell(&pos.reflect_vertical(), player);
                }
            }
        }
        reflected.last_move = self.last_move.map(|p| p.reflect_vertical());
        reflected
    }

    pub fn augmented(&self) -> Vec<Self> {
        let mut augmented = vec![self.clone()];
        
        // add 3 rotations
        for r in 1..4 {
            augmented.push(augmented[r-1].rotated_90());
        }
        // and their mirrors
        augmented.push(augmented[0].reflected_vertical());
        for r in 5..8 {
            augmented.push(augmented[r-1].rotated_90());
        }

        // 8 boards
        augmented
    }
}

impl fmt::Display for MainBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..9 {
            for x in 0..9 {
                let pos = XOPosition::new(x as u8, y as u8);
                let cell = self.get_cell(&pos);

                let last_move_mark = match self.last_move.clone() {
                    Some(last_move) => {
                        if last_move == pos {
                            "-"
                        } else {
                            " "
                        }
                    }
                    None => " ",
                };
                let p = match cell {
                    Some(player) => {
                        if player == XOPlayer::X {
                            player.to_string().red().to_string()
                        } else {
                            player.to_string().green().to_string()
                        }
                    }
                    None => " ".to_string(),
                };
                write!(f, "{last_move_mark}{}{last_move_mark}", p)?;
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
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl Default for MainBoard {
    fn default() -> Self {
        Self {
            small_boards: [SmallBoard::default(); 9],
            board: SmallBoard::default(),
            last_move: None,
        }
    }
}

pub struct BoardDisplayer {
    items: Vec<String>,
}

impl BoardDisplayer {
    pub fn new(items: Vec<String>) -> Self {
        if items.len() != 81 {
            panic!("Displayer expects 81 items, got {}", items.len())
        }
        Self { items }
    }
}

impl fmt::Display for BoardDisplayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..9 {
            for x in 0..9 {
                write!(f, "{}", self.items[x + 9 * y])?;
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
                writeln!(f)?;
            }
        }
        Ok(())
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
            Some(winner) => {
                println!("Player {winner} wins!");
                break Some(winner);
            }
            None => (),
        }
        player = player.other_player();
    }
}

#[test]
fn test_draw_by_filling_last_target_board() {
    let mut board = MainBoard::default();

    // Set up alternating wins in a pattern that doesn't result in a main board win
    // X wins
    for board_idx in [0, 2, 3, 7] {
        let large_pos = Position3::from_flat(board_idx);
        // Create diagonal win for X in each board
        board.set_cell(
            &XOPosition::from_subpos(large_pos.clone(), Position3::new(0, 0)),
            XOPlayer::X,
        );
        board.set_cell(
            &XOPosition::from_subpos(large_pos.clone(), Position3::new(1, 1)),
            XOPlayer::X,
        );
        board.set_cell(
            &XOPosition::from_subpos(large_pos.clone(), Position3::new(2, 2)),
            XOPlayer::X,
        );
    }

    // O wins
    for board_idx in [1, 5, 6, 8] {
        let large_pos = Position3::from_flat(board_idx);
        // Create vertical win for O in each board
        board.set_cell(
            &XOPosition::from_subpos(large_pos.clone(), Position3::new(1, 0)),
            XOPlayer::O,
        );
        board.set_cell(
            &XOPosition::from_subpos(large_pos.clone(), Position3::new(1, 1)),
            XOPlayer::O,
        );
        board.set_cell(
            &XOPosition::from_subpos(large_pos.clone(), Position3::new(1, 2)),
            XOPlayer::O,
        );
    }

    // Now board 4 (center) should be our target board - fill it almost completely
    let target_board_pos = Position3::from_flat(4);
    // Fill with mixed X and O moves, leaving only one space
    let moves = [
        (0, 0, XOPlayer::X),
        (0, 1, XOPlayer::O),
        (0, 2, XOPlayer::X),
        (1, 0, XOPlayer::O),
        (2, 0, XOPlayer::X),
        (2, 1, XOPlayer::O),
        (2, 2, XOPlayer::X),
        (1, 2, XOPlayer::O), // Leave (1,1) for final move
    ];

    for (x, y, player) in moves {
        board.set_cell(
            &XOPosition::from_subpos(target_board_pos.clone(), Position3::new(x, y)),
            player,
        );
    }

    // Set last move to force next move into center board
    board.set_cell(
        &XOPosition::from_subpos(Position3::from_flat(5), Position3::new(0, 2)),
        XOPlayer::O,
    );
    println!("{}", board);

    // Verify setup
    assert!(!board.is_draw(), "Board should not be in draw state yet");
    assert!(
        board.winner().is_none(),
        "There should be no winner before final move"
    );
    let valid_moves = board.valid_moves();
    assert_eq!(valid_moves.len(), 1, "Should only have one valid move left");

    // Make final move
    let final_move = XOPosition::from_subpos(target_board_pos, Position3::new(1, 1));
    board.set_cell(&final_move, XOPlayer::X);

    // Verify draw
    assert!(
        board.is_draw(),
        "Board should be in draw state after final move"
    );
    assert!(board.winner().is_none(), "There should be no winner");
}
