use crate::othello::{Game, Play, Player};
use rand::prelude::*;

const C_PARAM: f32 = 1.41; // sqrt(2)

/// Represents a node in the MCTS Tree.
pub struct Node {
    /// The index of the parent node in the `MCTSTree.arena`.
    parent: Option<usize>,
    /// The indexes of child nodes in the `MCTSTree.arena`.
    children: Vec<usize>,
    /// Vector of unexpanded moves.
    unexpanded_moves: Vec<Play>,

    // mcts members
    wins: f32,
    visits: u32,

    state: Game,
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

pub struct MctsSearchResult {
    pub search_iterations: u32,
}

/// Represents a MCTS Tree. Owns all the nodes in the tree.
pub struct Mcts {
    arena: Vec<Node>,
    /// The index of the root `Node`.
    root_node_index: usize,
}

impl Mcts {
    pub fn new(state: Game) -> Self {
        let mut arena: Vec<Node> = Vec::new();
        let node = Node::new(state, None);
        arena.push(node);

        return Mcts {
            arena,
            root_node_index: 0,
        };
    }

    /// Takes ownership of `node` and adds it to `self.arena`.
    /// # Arguments
    /// * `parent` - The index of the parent in `self.arena`.
    ///
    /// Returns the index of the newly added `Node`.
    fn add_node(&mut self, parent: usize, state: Game) -> usize {
        let index = self.arena.len();
        let node = Node::new(state, Some(parent)); // root node does not have parent
        self.arena.push(node);

        return index; // index of added node
    }

    fn get_node(&self, index: usize) -> &Node {
        &self.arena[index]
    }

    fn get_node_mut(&mut self, index: usize) -> &mut Node {
        &mut self.arena[index]
    }

    /// Clones `state` and mutates the game with `play`.
    fn advance_state(state: &Game, play: Play) -> Game {
        let mut tmp_state = state.clone();
        tmp_state.make_play(play);

        return tmp_state;
    }

    /// Returns the best child of the node at `self.arena[index]` according to uct formula or `None` if no `children`.
    fn select_best_child_uct(&self, index: usize) -> Option<usize> {
        let mut best_index: Option<usize> = None;
        let mut best_score = f32::MIN;

        let node = self.get_node(index);

        for child_index in &node.children {
            let child = self.get_node(*child_index);
            let score: f32 = (child.wins / child.visits as f32)
                + (C_PARAM * child.wins.log2().sqrt() / child.visits as f32);

            if score > best_score {
                best_index = Some(*child_index);
                best_score = score;
            }
        }

        return best_index;
    }

    /// ### Monte Carlo Tree Search - step 1.
    /// Returns the index of the selected node in `self.arena`.
    fn select(&self) -> usize {
        let mut node_index = self.root_node_index; // start at root node

        while self.get_node(node_index).is_fully_expanded()
            && !self.get_node(node_index).is_terminal()
        {
            let temp_node_index = self.select_best_child_uct(node_index);

            if let Some(index) = temp_node_index {
                node_index = index;
            } else {
                break; // leaf node found (no expanded nodes)
            }
        }

        return node_index;
    }

    /// ### Monte Carlo Tree Search - step 2.
    /// Picks `self.children[self.unexpanded_index]` and expands the node. Pops a `Play` from `unexpanded_plays` and pushes the index of the added `Node` to `children`.
    /// Returns the index of the new `Node`.
    ///
    /// # Panics
    /// This method panics if there are no more moves left to expand for the specified node.
    fn expand(&mut self, index: usize) -> usize {
        let last_move = self.get_node_mut(index).unexpanded_moves.pop();

        if let Some(play) = last_move {
            let new_node_index = self.arena.len();

            let new_state = Self::advance_state(&self.arena[index].state, play); // create new state from play
            self.add_node(index, new_state); // create new Node
            self.get_node_mut(index).children.push(new_node_index);

            return new_node_index;
        } else {
            panic!("No more moves left to expand.");
        }
    }

    /// ### Monte Carlo Tree Search - step 3.
    fn simulate(&self, index: usize) -> Player {
        let mut state = self.get_node(index).state.clone();

        let mut rng = thread_rng();

        while state.game_state() == Player::InProgress {
            let plays = state.generate_plays();
            // select random move
            let rand_index = rng.gen_range(0, plays.len());
            let play = plays[rand_index];

            state.make_play(play);

            if play == 64 {
                // check if other player has a move, if false, return Player::Tie
                if state.generate_plays()[0] == 64 {
                    // count number of pieces of each color
                    let black_count = state.black_pieces.count_ones();
                    let white_count = state.white_pieces.count_ones();

                    if black_count > white_count {
                        return Player::Black;
                    } else if white_count > black_count {
                        return Player::White;
                    } else {
                        return Player::Tie;
                    }
                }
            }
        }

        return state.game_state();
    }

    /// ### Monte Carlo Tree Search - step 4.
    fn backpropagate(&mut self, index: usize, winner: Player) {
        let node = self.get_node_mut(index);

        node.visits += 1;

        if node.state.player_to_move != winner {
            // is current player
            node.wins += 1.0;
        }

        if let Some(parent) = node.parent {
            self.backpropagate(parent, winner); // backpropagate parent
        }
    }

    /// Runs Monte Carlo Tree Search
    /// # Arguments
    /// * `time_budget` - the time budget for running the search in `ms`.
    pub fn run_search(&mut self, time_budget: u128) -> MctsSearchResult {
        use std::time::{Duration, Instant};

        let mut iterations_count: u32 = 0;
        let time_start = Instant::now();

        loop {
            let node_index = self.select(); // step 1
            if self.get_node(node_index).is_fully_expanded() {
                // step 2 skip
                let winner = self.simulate(node_index); // step 3
                self.backpropagate(node_index, winner); // step 4
            } else {
                let expanded_index = self.expand(node_index); // step 2
                let winner = self.simulate(expanded_index); // step 3
                self.backpropagate(expanded_index, winner); // step 4
            }

            iterations_count += 1;

            let duration: Duration = time_start.elapsed();
            if duration.as_millis() > time_budget {
                break;
            }
        }

        return MctsSearchResult {
            search_iterations: iterations_count,
        };
    }

    pub fn best_play(&self) -> Play {
        let root_node = self.get_node(self.root_node_index);

        if !root_node.is_fully_expanded() {
            panic!("Root is not fully expanded.");
        }

        let mut best_visits: u32 = 0;
        let mut best_play: Play = 0;

        for child_index in &root_node.children {
            let child = self.get_node(*child_index);

            if child.visits > best_visits {
                best_play = child.state.previous_move;
                best_visits = child.visits;
            }
        }

        return best_play;
    }
}
