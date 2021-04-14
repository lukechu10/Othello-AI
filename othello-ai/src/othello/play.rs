/// Represents the position on the game board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Play(pub u8);

impl Play {
    /// Create a new `Play` with specified `row` and `col`.
    pub fn new(row: u8, col: u8) -> Self {
        debug_assert!(row < 8);
        debug_assert!(col < 8);

        Self(row * 8 + col)
    }
}
