use crate::game::{Position, BlockPosition, FixedPointShort, Inventory};
use tokio::io::AsyncReadExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use std::pin::Pin;
use std::boxed::Box;
mod builder;
pub mod handler;
use flume::{Sender, Receiver};
use tokio::io::AsyncWriteExt;
use builder::ClassicPacketBuilder;
use anyhow::anyhow;
trait Packet {

}
pub trait PacketType {}
#[derive(Eq, PartialEq)]
pub enum ClientPacketTypes {
    LoginRequest,
    Handshake,
    ChatMessage,
    UnknownPacket,
    KeepAlive,
    PlayerPacket,
    PlayerPositionPacket,
    PlayerLookPacket,
    PlayerPositionAndLookPacket,
    EntityAction,
    Respawn,
    PlayerDigging,
    Animation,
    PlayerBlockPlacement,
    UseEntity,
    Disconnect,
    CloseWindow,
    HoldingChange,
    WindowClick,
    Transaction,
}
impl PacketType for ClientPacketTypes {

}
pub struct PacketReaderFancy<T: tokio::io::AsyncRead> {
    stream: Pin<Box<T>>,
    queue: Vec<ClientPacket>
}
impl <T: tokio::io::AsyncRead>PacketReaderFancy<T> {
    pub fn new(stream: Pin<Box<T>>) -> Self {
        Self { stream, queue: Vec::new() }
    }
    pub async fn read_generic(&mut self) -> anyhow::Result<ClientPacket> {
        //log::info!("Called read");
        if let Some(p) = self.queue.pop() {
            return Ok(p);
        } else {
            InternalReader::read(&mut self.stream).await
        }
    }
    pub async fn read(&mut self, packet_type: ClientPacketTypes) -> anyhow::Result<ClientPacket> {
        loop {
            for i in (0..self.queue.len()).rev() {
                if matches!(self.queue[i].packet_type(), packet_type) {
                    return Ok(self.queue.remove(i));
                }
            }
            self.queue.push(InternalReader::read(&mut self.stream).await?);
        }
    }
}
pub struct PacketWriter {
    stream: OwnedWriteHalf,
    recv: Receiver<ServerPacket>,
}
impl PacketWriter {
    pub fn new(stream: OwnedWriteHalf, recv: Receiver<ServerPacket>) -> Self {
        Self { stream, recv }
    }
    pub async fn run(mut self) -> anyhow::Result<()> {
        while let Ok(packet) = self.recv.recv_async().await {
            self.stream.write_all(&packet.as_bytes()?).await?;
        }
        Ok(())
    }
    pub async fn write(&mut self, packet: ServerPacket) -> anyhow::Result<()> {
        Ok(self.stream.write_all(&packet.as_bytes()?).await?)
    }
}
pub struct PacketReader {
    stream: PacketReaderFancy<OwnedReadHalf>,
    send: Sender<ClientPacket>,
}
impl PacketReader {
    pub fn new(stream: OwnedReadHalf, send: Sender<ClientPacket>) -> Self {
        Self { stream: PacketReaderFancy::new(Box::pin(stream)), send }
    }
    pub async fn run(mut self) -> anyhow::Result<()> {
        //log::info!("lOOP STRTED");
        loop {
            //log::info!("HI THERE");
            let packet = self.stream.read_generic().await?;
            if matches!(packet.packet_type(), ClientPacketTypes::UnknownPacket) {
                log::info!("unk knon");
                continue;
            }
            //log::info!("HELLO LOOK HERE");
            if let Err(_) = self.send.send_async(packet).await {
                log::info!("IT BROKE");
                return Ok(());
            }
        }
    }
    pub async fn read_generic(&mut self) -> anyhow::Result<ClientPacket> {
        self.stream.read_generic().await
    }
}
struct InternalReader<T: tokio::io::AsyncRead> {
    a: T
}
impl <T: tokio::io::AsyncRead>InternalReader<T> {
    pub async fn read(reader: &mut Pin<Box<T>>) -> anyhow::Result<ClientPacket> {
        //log::info!("Called!");
        let id = Self::read_byte_raw(reader).await?;
        match id {
            0x01 => {
                //log::info!("Login request!");
                return Ok(ClientPacket::LoginRequest( LoginRequest { protocol_version: Self::read_int(reader).await?, username: Self::read_string16(reader).await?, map_seed: Self::read_long(reader).await?, dimension: Self::read_byte(reader).await?}));
            }
            0x02 => {
                //log::info!("Handshake!");
                return Ok(ClientPacket::Handshake( Handshake { username: Self::read_string16(reader).await? }));
            }
            0x03 => {
                //log::info!("Chat!");
                return Ok(ClientPacket::ChatMessage( ChatMessage { message: Self::read_string16(reader).await? }));
            }
            0x00 => {
                return Ok(ClientPacket::KeepAlive);
                //log::info!("Keepalive!");
            }
            0x0A => {
                return Ok(ClientPacket::PlayerPacket( PlayerPacket { on_ground: Self::read_byte_raw(reader).await? != 0 }));
            }
            0x0B => {
                return Ok(ClientPacket::PlayerPositionPacket( PlayerPositionPacket { x: Self::read_double(reader).await?, y: Self::read_double(reader).await?, stance: Self::read_double(reader).await?, z: Self::read_double(reader).await?, on_ground: Self::read_byte_raw(reader).await? != 0}));
            }
            0x0C => {
                return Ok(ClientPacket::PlayerLookPacket( PlayerLookPacket { yaw: Self::read_float(reader).await?, pitch: Self::read_float(reader).await?, on_ground: Self::read_byte_raw(reader).await? != 0}));
            }
            0x0D => {
                return Ok(ClientPacket::PlayerPositionAndLookPacket( PlayerPositionAndLookPacket { x: Self::read_double(reader).await?, y: Self::read_double(reader).await?, stance: Self::read_double(reader).await?, z: Self::read_double(reader).await?, yaw: Self::read_float(reader).await?, pitch: Self::read_float(reader).await?, on_ground: Self::read_byte_raw(reader).await? != 0}));
            }
            0x13 => {
                return Ok(ClientPacket::EntityAction( EntityAction { eid: Self::read_int(reader).await?, action: Self::read_byte(reader).await? }));
            }
            0x09 => {
                return Ok(ClientPacket::Respawn( Respawn { world: Self::read_byte(reader).await? }));
            }
            0x0E => {
                return Ok(ClientPacket::PlayerDigging( PlayerDigging { status: Self::read_byte(reader).await?, x: Self::read_int(reader).await?, y: Self::read_byte(reader).await?, z: Self::read_int(reader).await?, face: Self::read_byte(reader).await? }));
            }
            0x12 => {
                return Ok(ClientPacket::Animation( Animation { eid: Self::read_int(reader).await?, animate: Self::read_byte(reader).await? }));
            }
            0x0F => {
                let x = Self::read_int(reader).await?;
                let y = Self::read_byte(reader).await?;
                let z = Self::read_int(reader).await?;
                let direction = Self::read_byte(reader).await?;
                let block_or_item_id = Self::read_short(reader).await?;
                let mut amount = None;
                let mut damage = None;
                if block_or_item_id >= 0 {
                    amount = Some(Self::read_byte(reader).await?);
                    damage = Some(Self::read_short(reader).await?);
                }
                return Ok(ClientPacket::PlayerBlockPlacement( PlayerBlockPlacement { x, y, z, direction, block_or_item_id, amount, damage }));
            }
            0x07 => {
                return Ok(ClientPacket::UseEntity( UseEntity { user: Self::read_int(reader).await?, target: Self::read_int(reader).await?, left_click: Self::read_byte_raw(reader).await? != 0 }));
            }
            0xFF => {
                return Ok(ClientPacket::Disconnect( Disconnect { reason: Self::read_string16(reader).await? }));
            }
            0x65 => {
                return Ok(ClientPacket::CloseWindow( CloseWindow { window_id: Self::read_byte(reader).await? }));
            }
            0x10 => {
                let unsanitized_id = Self::read_short(reader).await?;
                let id = unsanitized_id.min(8);
                let id = id.max(0);
                return Ok(ClientPacket::HoldingChange( HoldingChange { slot_id: id }));   
            }
            0x66 => {
                let window_id = Self::read_byte(reader).await?;
                let slot = Self::read_short(reader).await?;
                let right_click = Self::read_byte(reader).await?;
                let action_number = Self::read_short(reader).await?;
                let shift = Self::read_byte(reader).await? != 0;
                let item_id = Self::read_short(reader).await?;
                let mut item_count = None;
                let mut item_uses = None;
                if item_id != -1 {
                    item_count = Some(Self::read_byte(reader).await?);
                    item_uses = Some(Self::read_short(reader).await?);
                }
                return Ok(ClientPacket::WindowClick( WindowClick { window_id, slot, right_click, action_number, shift, item_id, item_count, item_uses }));
            }
            0x6A => {
                return Ok(ClientPacket::Transaction( Transaction { window_id: Self::read_byte(reader).await?, action_number: Self::read_short(reader).await?, accepted: Self::read_byte_raw(reader).await? != 0 }));
            }
            id => {
                log::warn!("Unknown packet id {:x}!", id);
                //return Err(anyhow!("Unknown packet id {}!", id));
                return Ok(ClientPacket::UnknownPacket);
            }
        }
    }
    pub async fn read_double(reader: &mut Pin<Box<T>>) -> anyhow::Result<f64> {
        let mut byte = [0; 8];
        reader.read_exact(&mut byte).await?;
        Ok(f64::from_be_bytes(byte))
    }
    pub async fn read_float(reader: &mut Pin<Box<T>>) -> anyhow::Result<f32> {
        let mut byte = [0; 4];
        reader.read_exact(&mut byte).await?;
        Ok(f32::from_be_bytes(byte))
    }
    pub async fn read_byte_raw(reader: &mut Pin<Box<T>>) -> anyhow::Result<u8> {
        let mut byte = [0; 1];
        reader.read_exact(&mut byte).await?;
        Ok(byte[0])
    }
    pub async fn read_byte(reader: &mut Pin<Box<T>>) -> anyhow::Result<i8> {
        let mut byte = [0; 1];
        reader.read_exact(&mut byte).await?;
        Ok(byte[0] as i8)
    }
    pub async fn read_short(reader: &mut Pin<Box<T>>) -> anyhow::Result<i16> {
        let mut byte = [0; 2];
        reader.read_exact(&mut byte).await?;
        Ok(i16::from_be_bytes(byte))
    }
    pub async fn read_int(reader: &mut Pin<Box<T>>) -> anyhow::Result<i32> {
        let mut byte = [0; 4];
        reader.read_exact(&mut byte).await?;
        Ok(i32::from_be_bytes(byte))
    }
    pub async fn read_long(reader: &mut Pin<Box<T>>) -> anyhow::Result<i64> {
        let mut byte = [0; 8];
        reader.read_exact(&mut byte).await?;
        Ok(i64::from_be_bytes(byte))
    }
    pub async fn read_string(reader: &mut Pin<Box<T>>) -> anyhow::Result<String> {
        let mut byte = vec![0; Self::read_short(reader).await? as usize];
        reader.read_exact(&mut byte).await?;
        let string = String::from_utf8_lossy(&byte).to_string();
        Ok(string)
    }
    async fn read_string16<'a>(
        reader: &mut Pin<Box<T>>,
      ) -> anyhow::Result<String> {
        let len = Self::read_short(reader).await?;
        let mut shorts = vec![];
        for i in 0..len {
            shorts.push(Self::read_short(reader).await? as u16);
        }
        let string = String::from_utf16_lossy(shorts.as_slice()).to_string();
        return Ok(string);
    }

}
#[derive(Debug)]
pub struct LoginRequest {
    pub protocol_version: i32, pub username: String, pub map_seed: i64, pub dimension: i8,
}
#[derive(Debug)]
pub struct Handshake {
    pub username: String,
}
pub struct SetBlock {
    position: BlockPosition, mode: u8, block_type: u8
}
pub struct PositionAndOrientation {
    pub player_id: u8, pub position: Position,
}
pub struct ChatMessage {
    message: String
}
pub struct PlayerPacket {
    on_ground: bool,
}
pub struct PlayerPositionPacket {
    x: f64,
    y: f64,
    stance: f64,
    z: f64,
    on_ground: bool,
}
pub struct PlayerLookPacket {
    yaw: f32,
    pitch: f32,
    on_ground: bool,
}
pub struct PlayerPositionAndLookPacket {
    x: f64,
    y: f64,
    stance: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
}
pub struct EntityAction {
    eid: i32,
    action: i8,
}
pub struct Respawn {
    world: i8,
}
#[derive(Clone)]
pub struct PlayerDigging {
    pub status: i8,
    pub x: i32,
    pub y: i8,
    pub z: i32,
    pub face: i8,
}
pub struct Animation {
    eid: i32,
    animate: i8
}
#[derive(Clone)]
pub struct PlayerBlockPlacement {
    pub x: i32,
    pub y: i8,
    pub z: i32,
    pub direction: i8,
    pub block_or_item_id: i16,
    pub amount: Option<i8>,
    pub damage: Option<i16>,
}
pub struct UseEntity {
    user: i32,
    target: i32,
    left_click: bool,
}
pub struct Disconnect {
    reason: String,
}
pub struct CloseWindow {
    window_id: i8,
}
pub struct HoldingChange {
    slot_id: i16,
}
pub struct WindowClick {
    window_id: i8,
    slot: i16,
    right_click: i8,
    action_number: i16,
    shift: bool,
    item_id: i16,
    item_count: Option<i8>,
    item_uses: Option<i16>,
}
pub struct Transaction {
    window_id: i8,
    action_number: i16,
    accepted: bool,
}
pub enum ClientPacket {
    LoginRequest(LoginRequest),
    Handshake(Handshake),
    ChatMessage(ChatMessage),
    UnknownPacket,
    KeepAlive,
    PlayerPacket(PlayerPacket),
    PlayerPositionPacket(PlayerPositionPacket),
    PlayerLookPacket(PlayerLookPacket),
    PlayerPositionAndLookPacket(PlayerPositionAndLookPacket),
    EntityAction(EntityAction),
    Respawn(Respawn),
    PlayerDigging(PlayerDigging),
    Animation(Animation),
    PlayerBlockPlacement(PlayerBlockPlacement),
    UseEntity(UseEntity),
    Disconnect(Disconnect),
    CloseWindow(CloseWindow),
    HoldingChange(HoldingChange),
    WindowClick(WindowClick),
    Transaction(Transaction),
}
impl ClientPacket {
    pub fn packet_type(&self) -> ClientPacketTypes {
        match self {
            ClientPacket::EntityAction { .. } => {
                ClientPacketTypes::EntityAction
            }
            ClientPacket::LoginRequest { .. } => {
                ClientPacketTypes::LoginRequest
            },
            ClientPacket::Handshake { .. } => {
                ClientPacketTypes::Handshake
            }
            ClientPacket::ChatMessage { .. } => {
                ClientPacketTypes::ChatMessage
            }
            ClientPacket::UnknownPacket => {
                ClientPacketTypes::UnknownPacket
            }
            ClientPacket::KeepAlive => {
                ClientPacketTypes::KeepAlive
            }
            ClientPacket::PlayerPacket { .. } => {
                ClientPacketTypes::PlayerPacket
            }
            ClientPacket::PlayerPositionPacket { .. } => {
                ClientPacketTypes::PlayerPositionPacket
            }
            ClientPacket::PlayerLookPacket { .. } => {
                ClientPacketTypes::PlayerLookPacket
            }
            ClientPacket::PlayerPositionAndLookPacket { .. } => {
                ClientPacketTypes::PlayerPositionAndLookPacket
            }
            ClientPacket::Respawn { .. } => {
                ClientPacketTypes::Respawn
            }
            ClientPacket::PlayerDigging { .. } => {
                ClientPacketTypes::PlayerDigging
            }
            ClientPacket::Animation { .. } => {
                ClientPacketTypes::Animation
            }
            ClientPacket::PlayerBlockPlacement { .. } => {
                ClientPacketTypes::PlayerBlockPlacement
            }
            ClientPacket::UseEntity { .. } => {
                ClientPacketTypes::UseEntity
            }
            ClientPacket::Disconnect { .. } => {
                ClientPacketTypes::Disconnect
            }
            ClientPacket::CloseWindow { .. } => {
                ClientPacketTypes::CloseWindow
            }
            ClientPacket::HoldingChange { .. } => {
                ClientPacketTypes::HoldingChange
            }
            ClientPacket::WindowClick { .. } => {
                ClientPacketTypes::WindowClick
            }
            ClientPacket::Transaction { .. } => {
                ClientPacketTypes::Transaction
            }
        }
    }
}
#[derive(Clone, Debug)]
pub enum ServerPacket {
    ChatMessage { message: String },
    ServerLoginRequest { entity_id: i32, unknown: String, unknown_2: String, map_seed: i64, dimension: i8 },
    Handshake { connection_hash: String },
    PreChunk { x: i32, z: i32, mode: bool },
    MapChunk { x: i32, y: i16, z: i32, size_x: u8, size_y: u8, size_z: u8, compressed_size: i32, compressed_data: Vec<u8> },
    SpawnPosition { x: i32, y: i32, z: i32 },
    PlayerPositionAndLook { x: f64, stance: f64, y: f64, z: f64, yaw: f32, pitch: f32, on_ground: bool },
    KeepAlive,
    NamedEntitySpawn { eid: i32, name: String, x: i32, y: i32, z: i32, rotation: i8, pitch: i8, current_item: i16 },
    EntityTeleport { eid: i32, x: i32, y: i32, z: i32, yaw: i8, pitch: i8 },
    EntityAction { eid: i32, action: i8 },
    Animation { eid: i32, animate: u8 },
    UpdateHealth { health: i16 },
    Respawn { world: i8 },
    DestroyEntity { eid: i32 },
    Disconnect { reason: String },
    TimeUpdate { time: i64 },
    InvWindowItems { inventory: Inventory },
    EntityStatus { eid: i32, entity_status: i8 },
    SoundEffect { effect_id: i32, x: i32, y: i8, z: i32, sound_data: i32 },
    Transaction { window_id: i8, action_number: i16, accepted: bool },
    BlockChange { x: i32, y: i8, z: i32, block_type: i8, block_metadata: i8 },
    EntityEquipment { eid: i32, slot: i16, item_id: i16, damage: i16 },
    PlayerBlockPlacement { x: i32, y: i8, z: i32, direction: i8, block_or_item_id: i16, amount: i8, damage: i16 },
    EntityVelocity { eid: i32, velocity_x: i16, velocity_y: i16, velocity_z: i16 },
    EntityLook { eid: i32, yaw: i8, pitch: i8 },
    PickupSpawn { eid: i32, item: i16, count: i8, damage: i16, x: i32, y: i32, z: i32, rotation: i8, pitch: i8, roll: i8 },
    SetSlot { window_id: i8, slot: i16, item_id: i16, item_count: Option<i8>, item_uses: Option<i16> },
    CollectItem { collected_eid: i32, collector_eid: i32 },
    EntityRelativeMove { eid: i32, dX: i8, dY: i8, dZ: i8 },
    EntityLookAndRelativeMove { eid: i32, dX: i8, dY: i8, dZ: i8, yaw: i8, pitch: i8 },
    OpenWindow { window_id: i8, inventory_type: i8, window_title: String, num_slots: i8 },
    MobSpawn { eid: i32, m_type: i8, x: i32, y: i32, z: i32, yaw: i8, pitch: i8 },
}


impl ServerPacket {
    pub fn as_bytes(&self) -> anyhow::Result<Vec<u8>> {
        match self {
            ServerPacket::MobSpawn { eid, m_type, x, y, z, yaw, pitch } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*eid);
                builder.insert_byte(*m_type);
                builder.insert_int(*x);
                builder.insert_int(*y);
                builder.insert_int(*z);
                builder.insert_byte(*yaw);
                builder.insert_byte(*pitch);
                builder.insert_byte(0x7F);
                builder.build(0x18)
            }
            ServerPacket::OpenWindow { window_id, inventory_type, window_title, num_slots } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_byte(*window_id);
                builder.insert_byte(*inventory_type);
                builder.insert_string(&window_title);
                builder.insert_byte(*num_slots);
                builder.build(0x64)
            }
            ServerPacket::EntityLookAndRelativeMove { eid, dX, dY, dZ, yaw, pitch } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*eid);
                builder.insert_byte(*dX);
                builder.insert_byte(*dY);
                builder.insert_byte(*dZ);
                builder.insert_byte(*yaw);
                builder.insert_byte(*pitch);
                builder.build(0x21)
            }
            ServerPacket::EntityRelativeMove { eid, dX, dY, dZ } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*eid);
                builder.insert_byte(*dX);
                builder.insert_byte(*dY);
                builder.insert_byte(*dZ);
                builder.build(0x1F)
            }
            ServerPacket::CollectItem { collected_eid, collector_eid } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*collected_eid);
                builder.insert_int(*collector_eid);
                builder.build(0x16)
            }
            ServerPacket::SetSlot { window_id, slot, item_id, item_count, item_uses } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_byte(*window_id);
                builder.insert_short(*slot);
                builder.insert_short(*item_id);
                if item_count.is_some() && item_uses.is_some() {
                    builder.insert_byte(item_count.unwrap());
                    builder.insert_short(item_uses.unwrap());
                }
                builder.build(0x67)
            }
            ServerPacket::PickupSpawn { eid, item, count, damage, x, y, z, rotation, pitch, roll } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*eid);
                builder.insert_short(*item);
                builder.insert_byte(*count);
                builder.insert_short(*damage);
                builder.insert_int(*x);
                builder.insert_int(*y);
                builder.insert_int(*z);
                builder.insert_byte(*rotation);
                builder.insert_byte(*pitch);
                builder.insert_byte(*roll);
                builder.build(0x15)
            }
            ServerPacket::EntityLook { eid, yaw, pitch } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*eid);
                builder.insert_byte(*yaw);
                builder.insert_byte(*pitch);
                builder.build(0x20)
            }
            ServerPacket::EntityVelocity { eid, velocity_x, velocity_y, velocity_z } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*eid);
                builder.insert_short(*velocity_x);
                builder.insert_short(*velocity_y);
                builder.insert_short(*velocity_z);
                builder.build(0x1C)
            }
            ServerPacket::PlayerBlockPlacement { x, y, z, direction, block_or_item_id, amount, damage } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*x);
                builder.insert_byte(*y);
                builder.insert_int(*z);
                builder.insert_byte(*direction);
                builder.insert_short(*block_or_item_id);
                builder.insert_byte(*amount);
                builder.insert_short(*damage);
                builder.build(0x0F)
            }
            ServerPacket::EntityEquipment { eid, slot, item_id, damage } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*eid);
                builder.insert_short(*slot);
                builder.insert_short(*item_id);
                builder.insert_short(*damage);
                builder.build(0x05)
            }
            ServerPacket::BlockChange { x, y, z, block_type, block_metadata } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*x);
                builder.insert_byte(*y);
                builder.insert_int(*z);
                builder.insert_byte(*block_type);
                builder.insert_byte(*block_metadata);
                builder.build(0x35)
            }
            ServerPacket::Transaction { window_id, action_number, accepted } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_byte(*window_id);
                builder.insert_short(*action_number);
                builder.insert_byte_raw(*accepted as u8);
                builder.build(0x6A)
            }
            ServerPacket::SoundEffect { effect_id, x, y, z, sound_data } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*effect_id);
                builder.insert_int(*x);
                builder.insert_byte(*y);
                builder.insert_int(*z);
                builder.insert_int(*sound_data);
                builder.build(0x3D)
            }
            ServerPacket::EntityStatus { eid, entity_status } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*eid);
                builder.insert_byte(*entity_status);
                builder.build(0x26)
            }
            ServerPacket::InvWindowItems { inventory } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_byte(0);
                builder.insert_short(inventory.items.len() as i16);
                for i in 0..inventory.items.len() {
                    let item = inventory.items.get(&(i as i8)).unwrap();
                    if item.id == 0 {
                        builder.insert_short(-1);
                    } else {
                        builder.insert_short(item.id);
                        builder.insert_byte(item.count);
                        builder.insert_short(item.damage);
                    }
                }
                builder.build(0x68)
            }
            ServerPacket::TimeUpdate { time } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_long(*time);
                builder.build(0x04)
            }
            ServerPacket::Disconnect { reason } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_string16(reason);
                builder.build(0xFF)
            }
            ServerPacket::DestroyEntity { eid } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*eid);
                builder.build(0x1D)
            }
            ServerPacket::Respawn { world } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_byte(*world);
                builder.build(0x09)
            }
            ServerPacket::UpdateHealth { health } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_short(*health);
                builder.build(0x08)
            }
            ServerPacket::EntityAction { eid, action } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*eid);
                builder.insert_byte(*action);
                builder.build(0x13)
            }
            ServerPacket::Animation { eid, animate } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*eid);
                builder.insert_byte_raw(*animate);
                builder.build(0x12)
            }
            ServerPacket::EntityTeleport { eid, x, y, z, yaw, pitch } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*eid);
                //builder.insert_string16(&name);
                builder.insert_int(*x);
                builder.insert_int(*y);
                builder.insert_int(*z);
                builder.insert_byte(*yaw);
                builder.insert_byte(*pitch);
                //builder.insert_short(*current_item);
                builder.build(0x22)
            }
            ServerPacket::NamedEntitySpawn { eid, name, x, y, z, rotation, pitch, current_item } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*eid);
                builder.insert_string16(&name);
                builder.insert_int(*x);
                builder.insert_int(*y);
                builder.insert_int(*z);
                builder.insert_byte(*rotation);
                builder.insert_byte(*pitch);
                builder.insert_short(*current_item);
                builder.build(0x14)
            }
            ServerPacket::ChatMessage { message } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_string16(&message);
                builder.build(0x03)
            }
            ServerPacket::KeepAlive => {
                let mut builder = ClassicPacketBuilder::new();
                builder.build(0x00)
            }
            ServerPacket::ServerLoginRequest { entity_id, unknown, unknown_2, map_seed, dimension } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*entity_id);
                builder.insert_string(&unknown);
                builder.insert_string(&unknown_2);
                builder.insert_long(*map_seed);
                builder.insert_byte(*dimension);
                builder.build(0x01)
            }
            ServerPacket::Handshake { connection_hash } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_string16(&connection_hash);
                builder.build(0x02)
            }
            ServerPacket::PreChunk { x, z, mode } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*x);
                builder.insert_int(*z);
                builder.insert_byte(*mode as i8);
                builder.build(0x32)
            }
            ServerPacket::MapChunk { x, y, z, size_x, size_y, size_z, compressed_size, compressed_data } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*x);
                builder.insert_short(*y);
                builder.insert_int(*z);
                builder.insert_byte_raw(*size_x);
                builder.insert_byte_raw(*size_y);
                builder.insert_byte_raw(*size_z);
                builder.insert_int(*compressed_size);
                builder.insert_bytearray(compressed_data.to_vec());
                builder.build(0x33)
            }
            ServerPacket::SpawnPosition { x, y, z } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_int(*x);
                builder.insert_int(*y);
                builder.insert_int(*z);
                builder.build(0x06)
            }
            ServerPacket::PlayerPositionAndLook { x, stance, y, z, yaw, pitch, on_ground } => {
                let mut builder = ClassicPacketBuilder::new();
                builder.insert_double(*x);
                builder.insert_double(*y);
                builder.insert_double(*stance);
                builder.insert_double(*z);
                builder.insert_float(*yaw);
                builder.insert_float(*pitch);
                builder.insert_byte(*on_ground as i8);
                builder.build(0x0D)
            }
        }
    } 
}
impl Packet for ServerPacket {}
impl Packet for ClientPacket {}