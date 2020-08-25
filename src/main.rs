mod othello;
use othello::game::Game;

fn main() {
    println!("Hello, world!");
    let mut game = Game::new();
    // println!("{}", game);
    // println!("{:?}", game.generate_plays());

    println!("{}", game);
    while game.has_valid_plays() {
        let play = game.generate_plays()[0];
        game.make_play(play);
        println!("{}", game);
    }
}
