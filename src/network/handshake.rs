use rand::RngCore;

use super::worker::Worker;
use super::packet::{ClientPacket, ClientPacketTypes, ServerPacket};
use super::ids::EntityID;
use crate::server::NewPlayer;
pub async fn handle_connection(worker: &mut Worker) -> anyhow::Result<NewPlayer> {
    let packet = worker.read().await?;
    if !matches!(packet.packet_type(), ClientPacketTypes::Handshake) {
        return Err(anyhow::anyhow!("Wrong packet!"));
    }
    let handshake_packet = if let ClientPacket::Handshake(packet) = packet {
        packet
    } else {
        return Err(anyhow::anyhow!("Wrong packet!"));
    };
    // log::info!("{} attempting to log in as {}", worker.addr, handshake_packet.username);
    //log::info!("Packet: {:?}", handshake_packet);
    let packet = ServerPacket::Handshake { connection_hash: "-".to_string() };
    worker.write(packet).await?;
    let packet = worker.read().await?;
    if !matches!(packet.packet_type(), ClientPacketTypes::LoginRequest) {
        return Err(anyhow::anyhow!("Wrong packet!"));
    }
    let lr_packet = if let ClientPacket::LoginRequest(packet) = packet {
        packet
    } else {
        return Err(anyhow::anyhow!("Wrong packet!"));
    };
    // log::info!("Successfully authenticated {}[/{}]", lr_packet.username, worker.addr);
    if lr_packet.protocol_version != 29 {
        worker.write(ServerPacket::Disconnect { reason: "Wrong version.".to_string() }).await?;
        return Err(anyhow::anyhow!("Wrong protocol version!"));
    }
    let id = EntityID::new();
    //log::info!("Packet: {:?}", lr_packet);
    //log::info!("[Connection worker] User {} logging in with entity id {}", lr_packet.username, id.0);
/*     let packet = ServerPacket::ServerLoginRequest { entity_id: id.0, unknown: "".to_string(), unknown_2: "".to_string(), map_seed: 0, dimension: 0};
    worker.write(packet).await?; */
    Ok(NewPlayer { username: lr_packet.username, recv_packets_recv: worker.recv_packets_recv.clone(), packet_send_sender: worker.packet_send_sender.clone(), id, addr: worker.addr})
}