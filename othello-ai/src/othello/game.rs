use crate::othello::play::Play;
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bitfield(pub u64);

/// Represents the state of a cell
#[derive(Debug, PartialEq)]
pub enum Cell {
    Empty,
    Black,
    White,
}

/// Represents the current state of the game.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Player {
    Black,
    White,
    InProgress,
    Tie,
}

/// Represents an Othello game board.
#[derive(Debug, Clone)]
pub struct Game {
    pub black_pieces: Bitfield,
    pub white_pieces: Bitfield,

    /// Next player to move
    pub player_to_move: Player,
    pub previous_move: Play,
}

impl Game {
    /// Creates a new blank game board.
    pub fn new() -> Self {
        Self {
            black_pieces: Bitfield((1 << Play::new(3, 3).0) | (1 << Play::new(4, 4).0)),
            white_pieces: Bitfield((1 << Play::new(3, 4).0) | (1 << Play::new(4, 3).0)),
            player_to_move: Player::Black,
            previous_move: Play(0),
        }
    }

    /// Shifts `disks` in the specified `Direction`.
    /// # Arguments
    /// * `disks` - The `BitField` to shift
    /// * `dir` - The `Direction` to shift the `BitField`
    fn shift(disks: &Bitfield, dir: u8) -> Bitfield {
        const MASKS: [u64; 8] = [
            0x7F7F7F7F7F7F7F7F, // Direction::Right
            0x007F7F7F7F7F7F7F, // Direction::DownRight
            0xFFFFFFFFFFFFFFFF, // Direction::Down
            0x00FEFEFEFEFEFEFE, // Direction::DownLeft
            0xFEFEFEFEFEFEFEFE, // Direction::Left
            0xFEFEFEFEFEFEFE00, // Direction::UpLeft
            0xFFFFFFFFFFFFFFFF, // Direction::Up
            0x7F7F7F7F7F7F7F00, // Direction::UpRight
        ];

        const LSHIFTS: [u64; 8] = [0, 0, 0, 0, 1, 9, 8, 7];
        const RSHIFTS: [u64; 8] = [1, 9, 8, 7, 0, 0, 0, 0];

        let dir_size = dir as usize;
        if dir < 4 {
            // shift right
            debug_assert!(LSHIFTS[dir_size] == 0, "Shifting right.");
            Bitfield((disks.0 >> RSHIFTS[dir_size]) & MASKS[dir_size])
        } else {
            // shift left
            debug_assert!(RSHIFTS[dir_size] == 0, "Shifting left.");
            Bitfield((disks.0 << LSHIFTS[dir_size]) & MASKS[dir_size])
        }
    }

    /// Returns a vector of moves. Generates moves for the player in `self.player_to_move`.
    fn generate_plays_bitfield(&self) -> Bitfield {
        let (my_disks, opponent_disks) = if self.player_to_move == Player::Black {
            (&self.black_pieces, &self.white_pieces)
        } else {
            (&self.white_pieces, &self.black_pieces)
        };

        let mut x: Bitfield;

        let empty_cells = !(my_disks.0 | opponent_disks.0); // opposite of union of my_disks and opponent_disks
        let mut legal_moves = 0; // initially has no moves

        debug_assert!(
            self.black_pieces.0 & self.white_pieces.0 == 0,
            "disk sets should be disjoint"
        );

        for dir in 0..8 {
            // perform 7 shifts in each direction and follow connected disks

            // get adjacent opponent disks
            x = Bitfield(Self::shift(my_disks, dir).0 & opponent_disks.0);

            // add opponent disks adjacent to those
            x.0 |= Self::shift(&x, dir).0 & opponent_disks.0;
            x.0 |= Self::shift(&x, dir).0 & opponent_disks.0;
            x.0 |= Self::shift(&x, dir).0 & opponent_disks.0;
            x.0 |= Self::shift(&x, dir).0 & opponent_disks.0;
            x.0 |= Self::shift(&x, dir).0 & opponent_disks.0;

            // empty cells adjacent to those are legal moves
            legal_moves |= Self::shift(&x, dir).0 & empty_cells;
        }

        debug_assert!(
            legal_moves & (self.black_pieces.0 | self.white_pieces.0) == 0,
            "Legal moves should not be on black or white pieces."
        );

        Bitfield(legal_moves)
    }

    /// Returns a `Vec<Play>` of legal plays.
    /// # Postcondition
    /// The returned vector always has a least 1 play. If there are no plays available, the method returns the "skip" play (represented by 64).
    pub fn generate_plays(&self) -> Vec<Play> {
        let mut bitfield = self.generate_plays_bitfield();

        let mut vec = Vec::new();
        vec.reserve(20);
        let mut index = 0;

        while bitfield.0 != 0 {
            if bitfield.0 % 2 == 1 {
                vec.push(Play(index));
            }
            bitfield.0 >>= 1;
            index += 1;
        }

        if vec.is_empty() {
            // add "skip" Play
            vec.push(Play(64)); // overflow
        }

        debug_assert!(!vec.is_empty());
        vec
    }

    /// Modifies game board and flips opponent disks.
    fn resolve_play(&mut self, play: Play) {
        let (my_disks, opponent_disks) = if self.player_to_move == Player::Black {
            (&mut self.black_pieces, &mut self.white_pieces)
        } else {
            (&mut self.white_pieces, &mut self.black_pieces)
        };

        let mut x: Bitfield;

        let new_disk = if play.0 == 64 {
            Bitfield(0) // error to overflow completely
        } else {
            Bitfield(1 << play.0)
        };
        let mut captured_disks: u64 = 0;

        debug_assert!(play.0 < 65, "move must be within the board"); // 64 is "skip" turn
        debug_assert!(
            my_disks.0 & opponent_disks.0 == 0,
            "disk sets must be disjoint"
        );
        debug_assert!(
            (my_disks.0 | opponent_disks.0) & new_disk.0 == 0,
            "target must be empty"
        );

        my_disks.0 |= new_disk.0; // mutate my_disks

        // flip opponent_disks
        for dir in 0..8 {
            // find opponent disk adjacent to new_disk
            x = Bitfield(Self::shift(&new_disk, dir).0 & opponent_disks.0);
            // follow adjacent disks
            x.0 |= Self::shift(&x, dir).0 & opponent_disks.0;
            x.0 |= Self::shift(&x, dir).0 & opponent_disks.0;
            x.0 |= Self::shift(&x, dir).0 & opponent_disks.0;
            x.0 |= Self::shift(&x, dir).0 & opponent_disks.0;
            x.0 |= Self::shift(&x, dir).0 & opponent_disks.0;

            // determine whether the disks were captured
            let bounding_disk = Self::shift(&x, dir).0 & my_disks.0;
            captured_disks |= if bounding_disk != 0 { x.0 } else { 0 }; // do nothing if bounding_disk == 0
        }

        // mutate board with captured_disks
        my_disks.0 ^= captured_disks;
        opponent_disks.0 ^= captured_disks;

        // flip player_to_move
        self.player_to_move = if self.player_to_move == Player::Black {
            Player::White
        } else {
            Player::Black
        };

        debug_assert!(
            (my_disks.0 & opponent_disks.0) == 0,
            "disk sets must still be disjoint"
        );
    }

    /// Mutates the board.
    pub fn make_play(&mut self, play: Play) {
        self.resolve_play(play);
        self.previous_move = play;
    }

    pub fn is_valid_play(&self, play: Play) -> bool {
        let plays = self.generate_plays_bitfield();

        let mask = 1 << play.0;

        plays.0 & mask != 0
    }

    /// Returns the `Cell` state with the specified `row` and `col`.
    pub fn cell_state(&self, row: u8, col: u8) -> Cell {
        let mask = 1 << Play::new(row, col).0;

        if self.black_pieces.0 & mask != 0 {
            Cell::Black
        } else if self.white_pieces.0 & mask != 0 {
            Cell::White
        } else {
            Cell::Empty
        }
    }

    /// Computes the game state.
    pub fn game_state(&self) -> Player {
        if !(self.black_pieces.0 | self.white_pieces.0) != 0 {
            Player::InProgress
        } else {
            // count number of pieces of each color
            let black_count = self.black_pieces.0.count_ones();
            let white_count = self.white_pieces.0.count_ones();

            match black_count.cmp(&white_count) {
                Ordering::Less => Player::White,
                Ordering::Equal => Player::Tie,
                Ordering::Greater => Player::Black,
            }
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cell::Empty => write!(f, "-"),
            Cell::Black => write!(f, "B"),
            Cell::White => write!(f, "W"),
        }
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..8 {
            for col in 0..8 {
                write!(f, "{}", self.cell_state(row, col))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
