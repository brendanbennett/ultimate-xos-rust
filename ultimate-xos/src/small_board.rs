use std::fmt;
use std::str::FromStr;
use sigmazero::game::Player;

#[derive(Debug, Clone, Copy)]
pub struct Board {
    bitboards: [u16; 2], // X: player 0, O: player 1, and last move
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum XOPlayer {
    #[default] X = 0,
    O = 1,
}

impl XOPlayer {
    const PLAYERS: [Self; 2] = [XOPlayer::X, XOPlayer::O];

    pub fn other_player(&self) -> XOPlayer {
        match self {
            XOPlayer::X => XOPlayer::O,
            XOPlayer::O => XOPlayer::X,
        }
    }
}

impl Player for XOPlayer {
    const PLAYERS: [Self; 2] = [XOPlayer::X, XOPlayer::O];

    fn other_player(&self) -> XOPlayer {
        match self {
            XOPlayer::X => XOPlayer::O,
            XOPlayer::O => XOPlayer::X,
        }
    }
}

impl fmt::Display for XOPlayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                XOPlayer::X => "X",
                XOPlayer::O => "O",
            }
        )
    }
}

const WINNING: [u16; 8] = [73, 73 << 1, 73 << 2, 7, 7 << 3, 7 << 6, 273, 84];

impl Board {
    pub fn set_cell(&mut self, position: &Position3, player: XOPlayer) {
        // Need to update next move
        let mask = (1 as u16) << (position.y * 3 + position.x);
        self.bitboards[player as usize] |= mask;
        self.bitboards[player.other_player() as usize] &= !mask;
    }

    pub fn get_cell(&self, position: &Position3) -> Option<XOPlayer> {
        let offset = position.y * 3 + position.x;
        let mask = (1 as u16) << offset;
        let is_player_x = mask & self.bitboards[XOPlayer::X as usize] != 0;
        let is_player_o = mask & self.bitboards[XOPlayer::O as usize] != 0;
        // println!("[{}, {}], board X: {:#018b}, mask: {:#018b}", bits[0], bits[1], self.bitboards[1], mask);
        if is_player_x && is_player_o {
            panic!("{position:?} set for both X and O")
        } else if is_player_o {
            Some(XOPlayer::O)
        } else if is_player_x {
            Some(XOPlayer::X)
        } else {
            None
        }
    }

    pub fn winner(&self) -> Option<XOPlayer> {
        for player in XOPlayer::PLAYERS {
            for win_case in WINNING {
                if !self.bitboards[player as usize] & win_case == 0 {
                    return Some(player);
                }
            }
        }
        None
    }

    fn count_player(&self, player: XOPlayer) -> u32 {
        (self.bitboards[player as usize] << 7).count_ones()
    }

    pub fn is_full(&self) -> bool {
        // Assumes board is valid
        self.count_player(XOPlayer::X) + self.count_player(XOPlayer::O) == 9
    }

    pub fn valid_moves(&self) -> Vec<Position3> {
        let valid_bits = !(self.bitboards[0] | self.bitboards[1]);
        let mut valid_moves: Vec<Position3> = Vec::new();
        if self.winner().is_some() {
            return valid_moves;
        }
        for i in 0..9 {
            if 1 & (valid_bits >> i) == 1 {
                valid_moves.push(Position3::new(i % 3, i / 3))
            }
        }
        valid_moves
    }
}

impl Default for Board {
    fn default() -> Self {
        Self { bitboards: [0, 0] }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..3 {
            for x in 0..3 {
                let cell = self.get_cell(&Position3::new(x as u8, y as u8));
                write!(f, " {} ", cell.map_or(" ".to_string(), |p| p.to_string()))?;
                if x == 0 || x == 1 {
                    write!(f, "|")?;
                }
            }
            if y == 0 || y == 1 {
                write!(f, "\n{}", "-".repeat(10))?
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Position3 {
    pub x: u8,
    pub y: u8,
}

impl Position3 {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x: x, y: y }
    }

    pub fn from_vec(vec: Vec<u32>) -> Result<Self, String> {
        if vec.len() != 2 {
            return Err("Incorrect number of coordinates".to_string());
        }
        Ok(Self {
            x: vec[0] as u8,
            y: vec[1] as u8,
        })
    }

    pub fn is_valid(&self) -> bool {
        if self.x > 2 || self.y > 2 {
            return false;
        }
        true
    }

    pub fn flat(&self) -> u8 {
        self.x + 3 * self.y
    }

    pub fn from_flat(flat_repr: u8) -> Self {
        Self {
            x: flat_repr % 3,
            y: flat_repr / 3,
        }
    }
}

impl FromStr for Position3 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split(",").map(|s| s.trim()).collect();
        if split.len() != 2 {
            return Err(format!(
                "Move requires 2 arguments, received {} with {}",
                s,
                split.len()
            )
            .to_string());
        }

        let x = split[0]
            .parse::<u8>()
            .map_err(|_| "Invalid x coordinate".to_string())?;
        let y = split[1]
            .parse::<u8>()
            .map_err(|_| "Invalid y coordinate".to_string())?;

        Ok(Position3::new(x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_default() {
        let b = Board::default();
        assert_eq!(b.bitboards, [0, 0]);
    }

    #[test]
    fn set_cell() {
        let mut b = Board::default();
        b.set_cell(&Position3::new(1, 1), XOPlayer::X);
        assert_eq!(b.bitboards[XOPlayer::X as usize], 16);
    }

    #[test]
    fn get_cell() {
        let b = Board { bitboards: [1, 0] };
        assert_eq!(b.get_cell(&Position3::new(0, 0)), Some(XOPlayer::X));
    }

    #[test]
    fn set_get_cell() {
        let mut b = Board::default();
        let player = XOPlayer::X;
        let pos = Position3::new(1, 1);
        b.set_cell(&pos, player);
        assert_eq!(b.get_cell(&pos), Some(player));
    }

    #[test]
    fn win() {
        let mut b = Board::default();
        b.set_cell(&Position3::new(0, 0), XOPlayer::X);
        b.set_cell(&Position3::new(2, 0), XOPlayer::X);
        b.set_cell(&Position3::new(1, 1), XOPlayer::X);
        b.set_cell(&Position3::new(1, 2), XOPlayer::X);
        b.set_cell(&Position3::new(0, 2), XOPlayer::X);
        b.set_cell(&Position3::new(1, 0), XOPlayer::O);
        b.set_cell(&Position3::new(0, 1), XOPlayer::O);
        b.set_cell(&Position3::new(2, 1), XOPlayer::O);
        b.set_cell(&Position3::new(2, 2), XOPlayer::O);
        assert_eq!(b.winner(), Some(XOPlayer::X));
    }

    #[test]
    fn reset_other_player() {
        let mut b = Board::default();
        let pos = Position3::new(1, 1);
        b.set_cell(&pos, XOPlayer::X);
        b.set_cell(&pos, XOPlayer::O);
        assert_eq!(b.get_cell(&pos), Some(XOPlayer::O));
    }

    #[test]
    fn test_count() {
        let mut b = Board::default();
        b.set_cell(&Position3::new(0, 1), XOPlayer::X);
        b.set_cell(&Position3::new(1, 0), XOPlayer::X);
        b.set_cell(&Position3::new(2, 2), XOPlayer::X);
        b.set_cell(&Position3::new(2, 0), XOPlayer::O);
        b.set_cell(&Position3::new(1, 1), XOPlayer::O);
        b.set_cell(&Position3::new(0, 2), XOPlayer::O);
        assert_eq!(b.count_player(XOPlayer::X), 3);
        assert_eq!(b.count_player(XOPlayer::O), 3);
    }

    #[test]
    fn test_valid_moves() {
        let mut b = Board::default();
        b.set_cell(&Position3::new(0, 1), XOPlayer::X);
        b.set_cell(&Position3::new(1, 0), XOPlayer::X);
        b.set_cell(&Position3::new(2, 2), XOPlayer::X);
        b.set_cell(&Position3::new(2, 0), XOPlayer::O);
        b.set_cell(&Position3::new(1, 1), XOPlayer::O);
        b.set_cell(&Position3::new(1, 2), XOPlayer::O);
        assert!(b.valid_moves().contains(&Position3::new(0, 0)));
        assert_eq!(b.valid_moves().len(), 3);
    }

    #[test]
    fn test_position_parse() {
        let _: Position3 = "1,2".parse().unwrap();
        assert!("13eq,3".parse::<Position3>().is_err());
    }
}
