mod worker;
pub mod packet;
pub mod handshake;
pub mod ids;
pub mod metadata;
pub mod message;
use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use crate::async_systems::chat::AsyncChatCommand;
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
    async_chat: Sender<AsyncChatCommand>,
}
impl Listener {
    pub async fn start_listening(new_players: Sender<NewPlayer>, async_chat: Sender<AsyncChatCommand>) -> anyhow::Result<()> {
        let addr = format!("{}:{}", CONFIGURATION.listen_address, CONFIGURATION.listen_port);
        log::info!("Listening on {}", addr);
        let listener = TcpListener::bind(&addr).await?;
        let listener = Listener {
            listener,
            new_players,
            async_chat,
        };
        tokio::task::spawn(async move {
            listener.run().await;
        });
        Ok(())
    }
    async fn run(mut self) {
        loop {
            if let Ok((stream, addr)) = self.listener.accept().await {
                log::info!("Connection from {:?}", addr);
                self.accept(stream, addr).await;
            }
        }
    }
    async fn accept(&mut self, stream: TcpStream, addr: SocketAddr) {
        let worker = Worker::new(stream, addr, self.new_players.clone(), self.async_chat.clone());
        worker.begin();
    }
}