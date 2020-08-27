/// Represents the position on the game board.
pub type Play = u8;

trait NewPlay {
    fn new_play(row: u8, col: u8) -> Play;
}

/// Create a new `Play` with specified `row` and `col`.
pub fn new_play(row: u8, col: u8) -> Play {
    debug_assert!(row < 8);
    debug_assert!(col < 8);

    row * 8 + col
}
