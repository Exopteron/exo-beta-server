use tokio::net::TcpStream;
use std::net::SocketAddr;
use flume::{Sender, Receiver};
use crate::server::NewPlayer;
use super::handshake;
use super::packet;
use super::packet::{PacketReader, PacketWriter};
use crate::network::packet::{ServerPacket, ClientPacket};
pub struct Worker {
    reader: PacketReader,
    writer: PacketWriter,
    addr: SocketAddr,
    new_players: Sender<NewPlayer>,
    pub packet_send_sender: Sender<ServerPacket>,
    pub recv_packets_recv: Receiver<ClientPacket>,
}
impl Worker {
    pub fn new(stream: TcpStream, addr: SocketAddr, new_players: Sender<NewPlayer>) -> Self {
        let (reader, writer) = stream.into_split();

        let (recv_packets_send, recv_packets_recv) = flume::unbounded();
        let (packet_send_sender, packet_send_recv) = flume::unbounded();
        let reader = PacketReader::new(reader, recv_packets_send);
        let writer = PacketWriter::new(writer, packet_send_recv);
        Self { reader, writer, addr, new_players, packet_send_sender, recv_packets_recv }
    }
    pub fn begin(self) {
        tokio::task::spawn(async move {
            self.run().await;
        });
    }
    async fn run(mut self) -> anyhow::Result<()> {
        match handshake::handle_connection(&mut self).await {
            Ok(res) => {
                log::info!("Sending");
                let user = res.username.clone();
                self.new_players.send_async(res).await?;
                self.do_main(user).await;
            }
            Err(e) => {
                log::error!("Error handling user: {:?}", e);
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
                log::debug!("{} lost connection: {:?}", username, e);
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