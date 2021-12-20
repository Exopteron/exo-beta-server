mod worker;
pub mod packet;
pub mod handshake;
pub mod ids;
pub mod metadata;
pub mod message;
use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use crate::player_count::PlayerCount;
use crate::server::NewPlayer;
use crate::configuration::CONFIGURATION;
// use crate::error::Result;
// never used flume before, looks cool and feather uses it
use flume::Sender;
use worker::Worker;
// this is rust's OOP
pub struct Listener {
    listener: TcpListener,
    new_players: Sender<NewPlayer>,
    player_count: PlayerCount
}
impl Listener {
    pub async fn start_listening(new_players: Sender<NewPlayer>, player_count: PlayerCount) -> anyhow::Result<()> {
        let addr = format!("{}:{}", CONFIGURATION.listen_address, CONFIGURATION.listen_port);
        log::info!("Listening on {}", addr);
        let listener = TcpListener::bind(&addr).await?;
        let listener = Listener {
            listener,
            new_players,
            player_count,
        };
        tokio::task::spawn(async move {
            listener.run().await;
        });
        Ok(())
    }
    async fn run(mut self) {
        loop {
            if let Ok((stream, addr)) = self.listener.accept().await {
                self.accept(stream, addr).await;
            }
        }
    }
    async fn accept(&mut self, stream: TcpStream, addr: SocketAddr) {
        let worker = Worker::new(stream, addr, self.new_players.clone(), self.player_count.clone());
        worker.begin();
    }
}