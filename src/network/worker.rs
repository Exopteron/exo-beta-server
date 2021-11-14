use tokio::net::TcpStream;
use std::net::SocketAddr;
use flume::{Sender, Receiver};
use crate::configuration::CONFIGURATION;
use crate::server::NewPlayer;
use super::handshake;
use super::packet;
use super::packet::{PacketReader, PacketWriter};
use crate::network::packet::{ServerPacket, ClientPacket};
pub struct Worker {
    reader: PacketReader,
    writer: PacketWriter,
    pub addr: SocketAddr,
    new_players: Sender<NewPlayer>,
    pub packet_send_sender: Sender<ServerPacket>,
    pub recv_packets_recv: Receiver<ClientPacket>,
}
impl Worker {
    pub fn new(stream: TcpStream, addr: SocketAddr, new_players: Sender<NewPlayer>) -> Self {
        let (reader, writer) = stream.into_split();

        let (recv_packets_send, recv_packets_recv) = flume::unbounded();
        let (packet_send_sender, packet_send_recv) = flume::unbounded();
        let reader = PacketReader::new(reader, recv_packets_send.clone());
        let writer = PacketWriter::new(writer, packet_send_recv.clone());
        Self { reader, writer, addr, new_players, packet_send_sender: packet_send_sender.clone(), recv_packets_recv: recv_packets_recv.clone() }
    }
    pub fn begin(self) {
        tokio::task::spawn(async move {
            self.run().await;
        });
    }
    async fn run(mut self) -> anyhow::Result<()> {
        match handshake::handle_connection(&mut self).await {
            Ok(res) => {
                //log::debug!("Sending");
                let user = res.username.clone();
                self.new_players.send_async(res).await?;
                self.do_main(user).await;
            }
            Err(e) => {
                log::error!("[Connection worker] Error handling user: {:?}", e);
            }
        }
        Ok(())
    }
    async fn do_main(self, username: String) {
        let Self {
            reader,
            writer,
            ..
        } = self;
        let reader = tokio::task::spawn(async move { reader.run().await });
        let writer = tokio::task::spawn(async move { writer.run().await });
        tokio::task::spawn(async move {
            let result = tokio::select!{
                a = reader => a,
                b = writer => b,
            };
            if let Err(e) = result {
                log::debug!("[Connection worker] {} lost connection: {:?}", username, e);
            }
        });
    }
    pub async fn read(&mut self) -> anyhow::Result<packet::ClientPacket> {
        self.reader.read_generic().await
    }
    pub async fn write(&mut self, packet: ServerPacket) -> anyhow::Result<()> {
        self.writer.write(packet).await
    } 
}