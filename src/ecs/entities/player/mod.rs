use flume::{Receiver, Sender};
use hecs::Entity;

use crate::{ecs::Ecs, game::Position, network::{ids::EntityID, packet::{ClientPacket, ServerPacket}}};

pub struct Player;
pub struct Username(pub String);
pub struct NetworkManager {
    pub recv_packets_recv: Receiver<ClientPacket>,
    pub packet_send_sender: Sender<ServerPacket>,
}
impl NetworkManager {
    pub fn new(r: Receiver<ClientPacket>, s: Sender<ServerPacket>) -> Self {
        Self { recv_packets_recv: r, packet_send_sender: s }
    }
}
pub struct PlayerBuilder {

}
impl PlayerBuilder {
    pub fn build(ecs: &mut Ecs, netmanager: NetworkManager, username: Username, position: Position, id: EntityID) -> Entity {
        ecs.spawn((Player, netmanager, position, username, id))
    }
}