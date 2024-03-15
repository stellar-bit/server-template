use futures::prelude::sink::SinkExt;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;
use stellar_bit_core::prelude::*;
use stellar_bit_core::{
    game::GameCmdExecutionError,
    network::{ClientRequest, ServerResponse},
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};

mod client_handle;
use client_handle::ClientHandle;

pub const SERVER_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 39453);

pub struct GameSession {
    pub game: Arc<RwLock<Game>>,
}

impl GameSession {
    pub fn new() -> Self {
        Self {
            game: Arc::new(RwLock::new(Game::new())),
        }
    }
    pub fn game_loop(self, fps: u32) -> Result<(), GameCmdExecutionError> {
        let target_duration = std::time::Duration::from_secs_f32(1. / fps as f32);

        loop {
            let frame_time_measure = std::time::Instant::now();

            let mut game = self.game.write().unwrap();

            let dt = now() - game.sync.last_update;
            game.update(dt.as_secs_f32());

            drop(game);

            let frame_time = frame_time_measure.elapsed();
            if frame_time < target_duration {
                std::thread::sleep(target_duration - frame_time);
            } else {
                eprintln!(
                    "Server is behind intended frame rate, delay: {} ms",
                    (frame_time - target_duration).as_millis()
                )
            }
        }
    }
}

pub async fn run_server(game: Arc<RwLock<Game>>) {
    let server_address = SERVER_ADDR;
    let listener = TcpListener::bind(server_address).await.unwrap();
    println!("Listening on address {}", server_address);
    while let Ok((stream, _)) = listener.accept().await {
        println!("Detected potential client");
        let game = game.clone();
        tokio::task::spawn(handle_client(stream, game));
    }
}

async fn handle_client(stream: TcpStream, game: Arc<RwLock<Game>>) {
    let mut client_handle = ClientHandle::new(stream, game).await.unwrap();
    loop {
        if let Err(err) = client_handle.update().await {
            eprintln!("{:?}", err);
            return;
        };
    }
}