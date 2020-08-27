use crate::othello::{Game, Play, Player};
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Represents a node in the MCTS Tree.
pub struct Node {
    /// The index of the parent node in the `MCTSTree.arena`.
    pub parent: Option<usize>,
    /// The indexes of child nodes in the `MCTSTree.arena`.
    pub children: Vec<usize>,
    /// Vector of unexpanded moves.
    pub unexpanded_moves: Vec<Play>,

    // mcts members
    pub wins: f32,
    pub visits: u32,

    pub state: Game,
}

impl Node {
    /// Creates a new `Node` with the specified `Game` state. `children` is generated automatically from `state`.
    pub fn new(state: Game, parent: Option<usize>) -> Self {
        let mut plays = state.generate_plays();

        // shuffle plays
        plays.shuffle(&mut thread_rng());

        Self {
            parent,
            children: Vec::new(),
            unexpanded_moves: plays,
            wins: 0.0,
            visits: 0,
            state,
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.state.game_state() != Player::InProgress
    }

    pub fn is_fully_expanded(&self) -> bool {
        self.unexpanded_moves.len() == 0
    }
}
