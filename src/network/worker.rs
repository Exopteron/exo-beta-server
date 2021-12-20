use super::handshake;
use super::handshake::HandshakeResult;
use super::packet;
use crate::configuration::CONFIGURATION;
use crate::player_count::PlayerCount;
use crate::protocol::ClientPlayPacket;
use crate::protocol::MinecraftCodec;
use crate::protocol::Readable;
use crate::protocol::ServerPlayPacket;
use crate::protocol::Writeable;
use crate::protocol::packets::server::Kick;
use crate::server::NewPlayer;
use flume::{Receiver, Sender};
use std::fmt::Debug;
use std::io;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::time::timeout;
pub struct Worker {
    reader: Reader,
    writer: Writer,
    pub addr: SocketAddr,
    new_players: Sender<NewPlayer>,
    pub packet_send_sender: Sender<ServerPlayPacket>,
    pub recv_packets_recv: Receiver<ClientPlayPacket>,
    pub player_count: PlayerCount
}
impl Worker {
    pub fn new(stream: TcpStream, addr: SocketAddr, new_players: Sender<NewPlayer>, player_count: PlayerCount) -> Self {
        let (reader, writer) = stream.into_split();

        let (recv_packets_send, recv_packets_recv) = flume::unbounded();
        let (packet_send_sender, packet_send_recv) = flume::unbounded();
        let reader = Reader::new(reader, recv_packets_send.clone());
        let writer = Writer::new(writer, packet_send_recv.clone());
        Self {
            reader,
            writer,
            addr,
            new_players,
            packet_send_sender: packet_send_sender.clone(),
            recv_packets_recv: recv_packets_recv.clone(),
            player_count,
        }
    }
    pub fn begin(self) {
        tokio::task::spawn(async move {
            self.run().await;
        });
    }
    async fn run(mut self) -> anyhow::Result<()> {
        match handshake::handle_connection(&mut self).await {
            Ok(res) => {
                if let HandshakeResult::Player(res) = res {
                    if self.player_count.try_add_player().is_err() {
                        self.write(ServerPlayPacket::Kick(Kick {
                            reason: "The server is full!".into(),
                        }))
                        .await
                        .ok();
                        return Ok(());
                    }
                    let user = res.username.clone();
                    self.new_players.send_async(res).await?;
                    self.do_main(user).await;
                }
            }
            Err(e) => {
                log::error!("[Connection worker] Error handling user: {:?}", e);
            }
        }
        Ok(())
    }
    async fn do_main(self, username: String) {
        let Self { reader, writer, .. } = self;
        let reader = tokio::task::spawn(async move { reader.run().await });
        let writer = tokio::task::spawn(async move { writer.run().await });
        tokio::task::spawn(async move {
            let result = tokio::select! {
                a = reader => a,
                b = writer => b,
            };
            if let Err(e) = result {
                log::debug!("[Connection worker] {} lost connection: {:?}", username, e);
            }
        });
    }
    pub async fn read<P: Readable>(&mut self) -> anyhow::Result<P> {
        self.reader.read().await
    }

    pub async fn write(&mut self, packet: impl Writeable + Debug) -> anyhow::Result<()> {
        self.writer.write(packet).await
    }
}

struct Reader {
    stream: OwnedReadHalf,
    codec: MinecraftCodec,
    buffer: [u8; 512],
    received_packets: Sender<ClientPlayPacket>,
}

impl Reader {
    pub fn new(stream: OwnedReadHalf, received_packets: Sender<ClientPlayPacket>) -> Self {
        Self {
            stream,
            codec: MinecraftCodec::new(),
            buffer: [0; 512],
            received_packets,
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            let packet = self.read::<ClientPlayPacket>().await?;
            let result = self.received_packets.send_async(packet).await;
            if result.is_err() {
                // server dropped connection
                return Ok(());
            }
        }
    }

    pub async fn read<P: Readable>(&mut self) -> anyhow::Result<P> {
        // Keep reading bytes and trying to get the packet.
        loop {
            if let Some(packet) = self.codec.next_packet::<P>()? {
                return Ok(packet);
            }

            let duration = Duration::from_secs(10);
            let read_bytes = timeout(duration, self.stream.read(&mut self.buffer)).await??;
            if read_bytes == 0 {
                return Err(io::Error::new(ErrorKind::UnexpectedEof, "read 0 bytes").into());
            }

            let bytes = &self.buffer[..read_bytes];
            self.codec.accept(bytes);
        }
    }
}

struct Writer {
    stream: OwnedWriteHalf,
    codec: MinecraftCodec,
    packets_to_send: Receiver<ServerPlayPacket>,
    buffer: Vec<u8>,
}

impl Writer {
    pub fn new(stream: OwnedWriteHalf, packets_to_send: Receiver<ServerPlayPacket>) -> Self {
        Self {
            stream,
            codec: MinecraftCodec::new(),
            packets_to_send,
            buffer: Vec::new(),
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        while let Ok(packet) = self.packets_to_send.recv_async().await {
            self.write(packet).await?;
        }
        Ok(())
    }

    pub async fn write(&mut self, packet: impl Writeable + Debug) -> anyhow::Result<()> {
        self.codec.encode(&packet, &mut self.buffer)?;
        self.stream.write_all(&self.buffer).await?;
        self.buffer.clear();
        Ok(())
    }
}

fn disconnected_message(e: anyhow::Error) -> String {
    if let Some(io_error) = e.downcast_ref::<io::Error>() {
        if io_error.kind() == ErrorKind::UnexpectedEof {
            return "disconnected".to_owned();
        }
    }
    format!("{:?}", e)
}
