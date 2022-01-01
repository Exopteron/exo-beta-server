use rand::RngCore;

use super::ids::NetworkID;
use super::worker::Worker;
use crate::configuration::CONFIGURATION;
use crate::protocol::io::{String16, PingData};
use crate::protocol::packets::server::{Handshake, LoginRequest, ServerHandshakePacket, Kick};
use crate::protocol::{ClientHandshakePacket, ClientLoginPacket, ServerPlayPacket};
use crate::server::NewPlayer;
pub enum HandshakeResult {
    Player(NewPlayer),
    Ping,
}
pub async fn handle_connection(worker: &mut Worker) -> anyhow::Result<HandshakeResult> {
    let packet = worker.read::<ClientHandshakePacket>().await?;
    if let ClientHandshakePacket::ServerListPing(_) = packet {
        handle_status(worker).await?;
        return Ok(HandshakeResult::Ping)
    } else if let ClientHandshakePacket::Handshake(handshake) = packet {
        log::info!(
            "{} attempting to log in as {}",
            worker.addr,
            handshake.username.0
        );
        //log::info!("Packet: {:?}", handshake_packet);
        let packet = ServerHandshakePacket::Handshake(Handshake {
            connection_hash: String16("-".to_owned()),
        });
        worker.write(packet).await?;
        let packet = worker.read::<ClientLoginPacket>().await?;
        let ClientLoginPacket::LoginRequest(login_request) = packet;
        // log::info!("Successfully authenticated {}[/{}]", lr_packet.username, worker.addr);
        if login_request.protocol_version != 17 {
            //worker.write(ServerPacket::Disconnect { reason: "Wrong version.".to_string() }).await?;
            return Err(anyhow::anyhow!("Wrong protocol version!"));
        }
        let id = NetworkID::new();
        Ok(HandshakeResult::Player(NewPlayer {
            username: login_request.username.0,
            recv_packets_recv: worker.recv_packets_recv.clone(),
            packet_send_sender: worker.packet_send_sender.clone(),
            id,
            addr: worker.addr,
        }))
    } else {
        unreachable!()
    }
}

async fn handle_status(worker: &mut Worker) -> anyhow::Result<()> {
    worker.write(ServerPlayPacket::PingData(PingData::new(CONFIGURATION.server_motd.clone(), worker.player_count.get() as usize, worker.player_count.get_max() as usize))).await?;
    Ok(())
}