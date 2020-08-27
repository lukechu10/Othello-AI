fn main() {
    use othello_ai::mcts::*;
    use othello_ai::othello::*;
    use rand::{thread_rng, Rng};

    println!("Hello, world!");
    let mut game = Game::new();

    println!("{}", game);
    while game.game_state() == Player::InProgress {
        let play = if game.player_to_move == Player::Black {
            // mcts ai
            let mut mcts_agent = Mcts::new(game.clone());
            let search_res = mcts_agent.run_search(100);
            println!("{} games simulated.", search_res.search_iterations);

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
