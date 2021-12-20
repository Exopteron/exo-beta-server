//! Packets sent from client to server.

use crate::protocol::Readable;

use super::*;

mod handshake;
mod login;
mod play;
mod status;

pub use handshake::*;
pub use login::*;
pub use play::*;
pub use status::*;
packet_enum!(ClientHandshakePacket {
    0x02 = Handshake,
    0xFE = ServerListPing
});

packet_enum!(ClientStatusPacket {
    0x00 = Request,
    0x01 = Ping,
});

packet_enum!(ClientLoginPacket {
    0x01 = LoginRequest,
});

packet_enum!(ClientPlayPacket {
    0x0D = PlayerPositionAndLook,
    0x00 = KeepAlive,
    0x0A = PlayerMovement,
    0x0B = PlayerPosition,
    0x0C = PlayerLook,
    0x03 = ChatMessage,
    0xFF = Disconnect,
    0x12 = Animation,
    0x13 = EntityAction,
    0x0E = PlayerDigging,
    0x6B = CreativeInventoryAction,
    0x10 = HoldingChange,
    0x0F = PlayerBlockPlacement,
    0x65 = CloseWindow,
    0x07 = UseEntity,
});