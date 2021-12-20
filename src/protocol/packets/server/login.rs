use crate::protocol::io::String16;

use super::*;

packets! {
    S {
        
    }
    LoginRequest {
        entity_id i32;
        not_used String16;
        map_seed i64;
        server_mode i32;
        dimension i8;
        difficulty i8;
        world_height u8;
        max_players u8;
    }

    Handshake {
        connection_hash String16;
    }
}
