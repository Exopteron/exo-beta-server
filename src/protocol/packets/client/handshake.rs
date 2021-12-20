use crate::protocol::io::String16;

packets! {
    Handshake {
        username String16;
    }

    LoginRequest {
        protocol_version i32;
        username String16;
        unused_1 i64;
        unused_2 i32;
        unused_3 i8;
        unused_4 i8;
        unused_5 u8;
        unused_6 u8;
    }

    ServerListPing {}
}
