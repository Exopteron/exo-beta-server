//! Packets sent from client to server.

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
});
