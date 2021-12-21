//! Packets sent from server to client;

use super::*;

mod login;
mod play;
mod status;

pub use login::*;
pub use play::*;
pub use status::*;
use crate::protocol::io::PingData;
packet_enum!(ServerStatusPacket {
    0x00 = Response,
    0x01 = Pong,
});
packet_enum!(ServerHandshakePacket {
    0x02 = Handshake,
});
// temporary
packet_enum!(ServerLoginPacket {
    0xFF = S,
});

packet_enum!(ServerPlayPacket {
    0x0D = PlayerPositionAndLook,
    0x06 = SpawnPosition,
    0x01 = LoginRequest,
    0x33 = ChunkData,
    0x32 = PreChunk,
    0x00 = KeepAlive,
    0x03 = ChatMessage,
    0xFF = Kick,
    0xFF = PingData,
    0x14 = NamedEntitySpawn,
    0x1D = DestroyEntity,
    0x1F = EntityRelativeMove,
    0x20 = EntityLook,
    0x21 = EntityLookAndRelativeMove,
    0x22 = EntityTeleport,
    0x28 = SendEntityMetadata,
    0x12 = SendEntityAnimation,
    0xC9 = PlayerListItem,
    0x68 = WindowItems,
    0x67 = SetSlot,
    0x6A = Transaction,
    0x05 = EntityEquipment,
    0x35 = BlockChange,
    0x3D = SoundEffect,
    0x08 = UpdateHealth,
    0x46 = NewState,
    0x09 = Respawn,
    0x26 = EntityStatus,
    0x04 = TimeUpdate,
});
