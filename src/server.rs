use flume::{Sender, Receiver};
use crate::network::packet::{ClientPacket, ServerPacket};
use crate::network::Listener;
use std::collections::HashMap;
use crate::game::Game;
use crate::network::ids::EntityID;
use std::sync::Arc;
use std::cell::RefCell;
use std::time::Duration;
use std::time::Instant;
pub struct NewPlayer {
    pub username: String,
    pub recv_packets_recv: Receiver<ClientPacket>,
    pub packet_send_sender: Sender<ServerPacket>,
    pub id: EntityID,
}
impl NewPlayer {
    pub async fn write(&mut self, packet: ServerPacket) -> anyhow::Result<()> {
        self.packet_send_sender.send_async(packet).await?;
        Ok(())
    }
    pub async fn read(&mut self) -> anyhow::Result<ClientPacket> {
        Ok(self.recv_packets_recv.recv_async().await?)
    }
}
pub struct Client {
    pub recv_packets_recv: Receiver<ClientPacket>,
    pub packet_send_sender: Sender<ServerPacket>,
    pub username: String,
    pub id: EntityID,
    pub all_clients: Arc<RefCell<HashMap<EntityID, Arc<RefCell<Client>>>>>,
}
impl Client {
    pub fn new(player: NewPlayer, id: EntityID, all_clients: Arc<RefCell<HashMap<EntityID, Arc<RefCell<Client>>>>>) -> Self {
        Self {
            recv_packets_recv: player.recv_packets_recv,
            packet_send_sender: player.packet_send_sender,
            username: player.username,
            id,
            all_clients,
        }
    }
    pub fn write(&mut self, packet: ServerPacket) -> anyhow::Result<()> {
        self.packet_send_sender.send(packet)?;
        Ok(())
    }
    pub async fn read(&mut self) -> anyhow::Result<ClientPacket> {
        Ok(self.recv_packets_recv.recv_async().await?)
    }
    pub fn recieved_packets(&self) -> impl Iterator<Item = ClientPacket> + '_ {
        self.recv_packets_recv.try_iter()
    }
    pub fn remove_self(&self) {
        let mut all = self.all_clients.borrow_mut();
        all.remove(&self.id);
    }
}
pub struct Server {
    new_players: Receiver<NewPlayer>,
    pub clients: Arc<RefCell<HashMap<EntityID, Arc<RefCell<Client>>>>>,
    pub last_ping_time: Instant,
}
impl Server {
    pub async fn bind() -> anyhow::Result<Self> {
        let (new_players_send, new_players) = flume::bounded(4);
        Listener::start_listening(new_players_send).await?;
        Ok( Self { new_players, clients: Arc::new(RefCell::new(HashMap::new())), last_ping_time: Instant::now() } )
    }
    pub fn register(self, game: &mut Game) {
        game.insert_object(self);
    }
    pub fn accept_clients(&mut self) -> Vec<EntityID> {
        let mut clients = Vec::new();
        for player in self.new_players.clone().try_iter() {
            let id = self.create_client(player);
            clients.push(id);
        }
        clients
    }
    fn create_client(&mut self, player: NewPlayer) -> EntityID {
        let id = player.id;
        let client = Client::new(player, id.clone(), self.clients.clone());
        self.clients.borrow_mut().insert(id.clone(), Arc::new(RefCell::new(client)));
        id
    }
    pub fn get_id(&mut self) -> EntityID {
        EntityID::new()
    }
    pub fn broadcast(&mut self, mut function: impl FnMut(&mut Client) -> anyhow::Result<()>) -> anyhow::Result<()> {
        for client in self.clients.borrow_mut().iter_mut() {
            function(&mut client.1.borrow_mut())?;
        }
        Ok(())
    }
}