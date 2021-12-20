use rand::RngCore;

use super::worker::Worker;
use super::ids::NetworkID;
use crate::protocol::io::String16;
use crate::protocol::packets::server::{Handshake, ServerHandshakePacket, LoginRequest};
use crate::protocol::{ClientHandshakePacket, ClientLoginPacket};
use crate::server::NewPlayer;
pub async fn handle_connection(worker: &mut Worker) -> anyhow::Result<NewPlayer> {
    log::info!("A");
    let packet = worker.read::<ClientHandshakePacket>().await?;
    log::info!("B");
    let ClientHandshakePacket::Handshake(handshake) = packet;
    log::info!("{} attempting to log in as {}", worker.addr, handshake.username.0);
    //log::info!("Packet: {:?}", handshake_packet);
    let packet = ServerHandshakePacket::Handshake(Handshake { connection_hash: String16("-".to_owned())});
    worker.write(packet).await?;
    log::info!("Written");
    let packet = worker.read::<ClientLoginPacket>().await?;
    log::info!("Sup");
    let ClientLoginPacket::LoginRequest(login_request) = packet;
    // log::info!("Successfully authenticated {}[/{}]", lr_packet.username, worker.addr);
    if login_request.protocol_version != 69420 && false {
        //worker.write(ServerPacket::Disconnect { reason: "Wrong version.".to_string() }).await?;
        return Err(anyhow::anyhow!("Wrong protocol version!"));
    }
    let id = NetworkID::new();
    Ok(NewPlayer { username: login_request.username.0, recv_packets_recv: worker.recv_packets_recv.clone(), packet_send_sender: worker.packet_send_sender.clone(), id, addr: worker.addr})
}