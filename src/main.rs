use stellar_bit_server_template::{run_server, GameSession};


#[tokio::main]
async fn main() {
    let game_session = GameSession::new();

    let game_clone = game_session.game.clone();
    tokio::task::spawn(run_server(game_clone));

    game_session.game_loop(200).unwrap();
}