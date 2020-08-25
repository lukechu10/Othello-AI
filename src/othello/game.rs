use crate::othello::play::{new_play, Play};

/// Alias for `u64`. A `BitField` is used for black locations and another for white locations.
type BitField = u64;

/// Represents a player.
#[derive(Debug, PartialEq)]
pub enum Player {
    Black,
    White,
}

/// Represents the state of a cell
#[derive(Debug, PartialEq)]
pub enum Cell {
    Empty,
    Black,
    White,
}

/// Represents an Othello game board.
#[derive(Debug)]
pub struct Game {
    pub black_pieces: BitField,
    pub white_pieces: BitField,

    /// Next player to move
    player_to_move: Player,
}

impl Game {
    /// Creates a new blank game board.
    pub fn new() -> Self {
        return Self {
            black_pieces: (1 << new_play(3, 3)) | (1 << new_play(4, 4)),
            white_pieces: (1 << new_play(3, 4)) | (1 << new_play(4, 3)),
            player_to_move: Player::Black,
        };
    }

    /// Shifts `disks` in the specified `Direction`.
    /// # Arguments
    /// * `disks` - The `BitField` to shift
    /// * `dir` - The `Direction` to shift the `BitField`
    fn shift(disks: &BitField, dir: u8) -> BitField {
        const MASKS: [u64; 8] = [
            0x7F7F7F7F7F7F7F7F, // Direction::Right
            0x007F7F7F7F7F7F7F, // Direction::DownRight
            0xFFFFFFFFFFFFFFFF, // Direction::Down
            0x00FEFEFEFEFEFEFE, // Direction::DownLeft
            0xFEFEFEFEFEFEFEFE, // Direction::Left
            0xFEFEFEFEFEFEFE0E, // Direction::UpLeft
            0xFFFFFFFFFFFFFFFF, // Direction::Up
            0x7F7F7F7F7F7F7F0F, // Direction::UpRight
        ];

        const LSHIFTS: [u64; 8] = [0, 0, 0, 0, 1, 9, 8, 7];
        const RSHIFTS: [u64; 8] = [1, 9, 8, 7, 0, 0, 0, 0];

        let dir_size = dir as usize;
        if dir < 4 {
            // shift right
            debug_assert!(LSHIFTS[dir_size] == 0, "Shifting right.");
            return (disks >> RSHIFTS[dir_size]) & MASKS[dir_size];
        } else {
            // shift left
            debug_assert!(RSHIFTS[dir_size] == 0, "Shifting left.");
            return (disks << LSHIFTS[dir_size]) & MASKS[dir_size];
        }
    }

    /// Returns a vector of moves. Generates moves for the player in `self.player_to_move`.
    fn generate_plays_bitfield(&self) -> BitField {
        let my_disks: &BitField;
        let opponent_disks: &BitField;
        if self.player_to_move == Player::Black {
            my_disks = &self.black_pieces;
            opponent_disks = &self.white_pieces;
        } else {
            my_disks = &self.white_pieces;
            opponent_disks = &self.black_pieces;
        }

        let mut x: BitField;

        let empty_cells: BitField = !(my_disks | opponent_disks); // opposite of union of my_disks and opponent_disks
        let mut legal_moves: BitField = 0; // initially has no moves

        assert_eq!(
            self.black_pieces & self.white_pieces,
            0,
            "Disk sets should be disjoint."
        );

        for dir in 0..8 {
            // perform 7 shifts in each direction and follow connected disks

            // get adjacent opponent disks
            x = Self::shift(my_disks, dir) & opponent_disks;

            // add opponent disks adjacent to those
            x |= Self::shift(&x, dir) & opponent_disks;
            x |= Self::shift(&x, dir) & opponent_disks;
            x |= Self::shift(&x, dir) & opponent_disks;
            x |= Self::shift(&x, dir) & opponent_disks;
            x |= Self::shift(&x, dir) & opponent_disks;

            // empty cells adjacent to those are legal moves
            legal_moves |= Self::shift(&x, dir) & empty_cells;
        }

        return legal_moves;
    }

    pub fn generate_plays(&self) -> Vec<Play> {
        let mut bitfield: BitField = self.generate_plays_bitfield();

        let mut v = Vec::new();
        let mut index: u8 = 0;

        while bitfield != 0 {
            if bitfield % 2 == 1 {
                v.push(index);
            }
            bitfield >>= 1;
            index += 1;
        }

        v
    }

    /// Checks if `self.player_to_move` has any valid moves this turn.
    pub fn has_valid_plays(&self) -> bool {
        self.generate_plays_bitfield() != 0
    }

    // pub fn is_valid_move(&self) {}

    /// Modifies game board and flips opponent disks.
    fn resolve_play(&mut self, play: Play) {
        let my_disks: &mut BitField;
        let opponent_disks: &mut BitField;
        if self.player_to_move == Player::Black {
            my_disks = &mut self.black_pieces;
            opponent_disks = &mut self.white_pieces;
        } else {
            my_disks = &mut self.white_pieces;
            opponent_disks = &mut self.black_pieces;
        }

        let mut x: u64;

        let new_disk: u64 = 1 << play; // shift 1 to correct index
        let mut captured_disks: u64 = 0;

        debug_assert!(play < 64, "Move must be within the board.");
        debug_assert!(
            *my_disks & *opponent_disks == 0,
            "Disk sets must be disjoint."
        );
        debug_assert!(
            (*my_disks | *opponent_disks) & new_disk == 0,
            "Target must be empty."
        );

        *my_disks |= new_disk; // mutate my_disks

        // flip opponent_disks
        for dir in 0..8 {
            // find opponent disk adjacent to new_disk
            x = Self::shift(&new_disk, dir) & *opponent_disks;
            // follow adjacent disks
            x |= Self::shift(&x, dir) & *opponent_disks;
            x |= Self::shift(&x, dir) & *opponent_disks;
            x |= Self::shift(&x, dir) & *opponent_disks;
            x |= Self::shift(&x, dir) & *opponent_disks;
            x |= Self::shift(&x, dir) & *opponent_disks;

            // determine whether the disks were captured
            let bounding_disk = Self::shift(&x, dir) & *my_disks;
            captured_disks |= if bounding_disk != 0 { x } else { 0 }; // do nothing if bounding_disk == 0
        }

        debug_assert!(captured_disks != 0, "A valid move must capture disks.");

        // mutate board with captured_disks
        *my_disks ^= captured_disks;
        *opponent_disks ^= captured_disks;

        // flip player_to_move
        self.player_to_move = if self.player_to_move == Player::Black {
            Player::White
        } else {
            Player::Black
        };

        debug_assert!(
            (*my_disks & *opponent_disks) == 0,
            "The sets must still be disjoint"
        );
    }

    /// Makes sure `play` is a valid `Play` and mutates the board.
    pub fn make_play(&mut self, play: Play) {
        // debug_assert!
        self.resolve_play(play);
    }

    /// Returns the `Cell` state with the specified `row` and `col`.
    pub fn cell_state(&self, row: u8, col: u8) -> Cell {
        let mask: u64 = 1 << new_play(row, col);

        if self.black_pieces & mask != 0 {
            return Cell::Black;
        } else if self.white_pieces & mask != 0 {
            return Cell::White;
        } else {
            return Cell::Empty;
        }
    }
}

use std::fmt;
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
            writeln!(f, "")?;
        }
        Ok(())
    }
}
