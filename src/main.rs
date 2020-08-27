#![allow(dead_code)]
#![allow(unused_variables)]

mod node;
mod othello;

use node::Mcts;
use othello::game::{Game, Player};
use othello::play::Play;
use rand::{thread_rng, Rng};

fn main() {
    println!("Hello, world!");
    let mut game = Game::new();
    // println!("{}", game);
    // println!("{:?}", game.generate_plays());

    println!("{}", game);
    while game.game_state() == Player::InProgress {
        let play = if game.player_to_move == Player::Black {
            // mcts ai
            let mut mcts_agent = Mcts::new(game.clone());
            mcts_agent.run_search();

            mcts_agent.best_play()
        } else {
            // random ai
            let plays = game.generate_plays();
            let mut rng = thread_rng();
            let rand_index = rng.gen_range(0, plays.len());

            plays[rand_index]
        };
        game.make_play(play);
        println!("{}", game);
    }

    println!("{:?} wins", game.game_state());
}
