use flume::{Sender, Receiver};
use crate::network::Listener;
use crate::protocol::ClientPlayPacket;
use crate::protocol::ServerPlayPacket;
use std::collections::HashMap;
use crate::game::Game;
use crate::network::ids::NetworkID;
use std::sync::Arc;
use std::cell::RefCell;
use std::time::Duration;
use std::time::Instant;
use std::net::*;
pub struct NewPlayer {
    pub username: String,
    pub recv_packets_recv: Receiver<ClientPlayPacket>,
    pub packet_send_sender: Sender<ServerPlayPacket>,
    pub id: NetworkID,
    pub addr: SocketAddr,
}
impl NewPlayer {
    pub async fn write(&mut self, packet: ServerPlayPacket) -> anyhow::Result<()> {
        self.packet_send_sender.send_async(packet).await?;
        Ok(())
    }
    pub async fn read(&mut self) -> anyhow::Result<ClientPlayPacket> {
        Ok(self.recv_packets_recv.recv_async().await?)
    }
}
pub struct Client {
    pub recv_packets_recv: Receiver<ClientPlayPacket>,
    pub packet_send_sender: Sender<ServerPlayPacket>,
    pub username: String,
    pub id: NetworkID,
    pub addr: SocketAddr,
}
impl Client {
    pub fn new(player: NewPlayer, id: NetworkID) -> Self {
        Self {
            recv_packets_recv: player.recv_packets_recv,
            packet_send_sender: player.packet_send_sender,
            username: player.username,
            id,
            addr: player.addr,
        }
    }
    pub fn write(&mut self, packet: ServerPlayPacket) -> anyhow::Result<()> {
        self.packet_send_sender.send(packet)?;
        Ok(())
    }
    pub async fn read(&mut self) -> anyhow::Result<ClientPlayPacket> {
        Ok(self.recv_packets_recv.recv_async().await?)
    }
    pub fn recieved_packets(&self) -> impl Iterator<Item = ClientPlayPacket> + '_ {
        self.recv_packets_recv.try_iter()
    }
}
pub struct Server {
    new_players: Receiver<NewPlayer>,
    pub clients: HashMap<NetworkID, Client>,
    pub last_ping_time: Instant,
}
impl Server {
    pub async fn bind() -> anyhow::Result<Self> {
        let (new_players_send, new_players) = flume::bounded(4);
        Listener::start_listening(new_players_send).await?;
        Ok( Self { new_players, clients: HashMap::new(), last_ping_time: Instant::now() } )
    }
    pub fn register(self, game: &mut Game) {
        game.insert_object(self);
    }
    pub fn accept_clients(&mut self) -> Vec<NetworkID> {
        let mut clients = Vec::new();
        for player in self.new_players.clone().try_iter() {
            let id = self.create_client(player);
            clients.push(id);
        }
        clients
    }
    fn create_client(&mut self, player: NewPlayer) -> NetworkID {
        let id = player.id;
        let client = Client::new(player, id.clone());
        self.clients.insert(id.clone(), client);
        id
    }
    pub fn get_id(&mut self) -> NetworkID {
        NetworkID::new()
    }
    pub fn broadcast(&mut self, mut function: impl FnMut(&mut Client) -> anyhow::Result<()>) -> anyhow::Result<()> {
        for mut client in self.clients.iter_mut() {
            function(&mut client.1)?;
        }
        Ok(())
    }
}