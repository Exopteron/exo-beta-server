//! Packets sent from server to client;

use super::*;

mod login;
mod play;
mod status;

pub use login::*;
pub use play::*;
pub use status::*;

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
});
