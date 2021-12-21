use anyhow::bail;

pub use chunk_data::{ChunkData, ChunkDataKind};
use crate::{protocol::{Readable, Writeable, io::{String16, AbsoluteInt, Slot}}, entities::metadata::EntityMetadata, network::metadata::Metadata, world::chunks::BlockState, game::BlockPosition};

use super::*;

mod chunk_data;
packets! {
    PlayerPositionAndLook {
        x f64;
        y f64;
        stance f64;
        z f64;
        yaw f32;
        pitch f32;
        on_ground bool;
    }
    
    SpawnPosition {
        x i32;
        y i32;
        z i32;
    }

    PreChunk {
        chunk_x i32;
        chunk_z i32;
        mode bool;
    }

    KeepAlive {
        id i32;
    }
    ChatMessage {
        message String16;
    }
    Kick {
        reason String16;
    }

    NamedEntitySpawn {
        eid i32;
        player_name String16;
        x AbsoluteInt;
        y AbsoluteInt;
        z AbsoluteInt;
        rotation i8;
        pitch i8;
        current_item i16;
    }

    DestroyEntity {
        eid i32;
    }

    EntityRelativeMove {
        eid i32;
        delta_x i8;
        delta_y i8;
        delta_z i8;
    }

    EntityLook {
        eid i32;
        yaw i8;
        pitch i8;
    }

    EntityLookAndRelativeMove {
        eid i32;
        delta_x i8;
        delta_y i8;
        delta_z i8;
        yaw i8;
        pitch i8;
    }

    EntityTeleport {
        eid i32;
        x AbsoluteInt;
        y AbsoluteInt;
        z AbsoluteInt;
        yaw i8;
        pitch i8;
    }

    SendEntityMetadata {
        eid i32;
        metadata Metadata;
    }

    SendEntityAnimation {
        eid i32;
        animation EntityAnimationType;
    }

    PlayerListItem {
        name String16;
        online bool;
        ping i16;
    }

    WindowItems {
        window_id i8;
        items ShortPrefixedVec<Slot>;
    }

    SetSlot {
        window_id i8;
        slot i16;
        item Slot;
    }

    Transaction {
        window_id i8;
        action_number i16;
        accepted bool;
    }

    EntityEquipment {
        eid i32;
        slot i16;
        item_id i16;
        damage i16;
    }

    BlockChange {
        pos BlockPosition;
        state BlockState;
    }

    SoundEffect {
        effect SoundEffectKind;
        pos BlockPosition;
        data i32;
    }

    UpdateHealth {
        health i16;
        food i16;
        saturation f32;
    }

    NewState {
        reason i8;
        gamemode i8;
    }

    Respawn {
        world i8;
        difficulty i8;
        gamemode i8;
        world_height i16;
        map_seed i64;
    }

    EntityStatus {
        eid i32;
        status EntityStatusKind;
    }

    TimeUpdate {
        time i64;
    }
}