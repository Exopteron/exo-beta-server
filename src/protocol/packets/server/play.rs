use anyhow::bail;

pub use chunk_data::{ChunkData, ChunkDataKind};
use crate::{protocol::{Readable, Writeable, io::{String16, AbsoluteInt, Slot, RotationFraction360}}, entities::metadata::EntityMetadata, network::metadata::Metadata, world::chunks::BlockState, game::BlockPosition};

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
        yaw RotationFraction360;
        pitch RotationFraction360;
    }

    EntityLookAndRelativeMove {
        eid i32;
        delta_x i8;
        delta_y i8;
        delta_z i8;
        yaw RotationFraction360;
        pitch RotationFraction360;
    }

    EntityTeleport {
        eid i32;
        x AbsoluteInt;
        y AbsoluteInt;
        z AbsoluteInt;
        yaw RotationFraction360;
        pitch RotationFraction360;
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

    UpdateSign {
        x i32;
        y i16;
        z i32;
        text1 String16;
        text2 String16;
        text3 String16;
        text4 String16;
    }

    BlockAction {
        x i32;
        y i16;
        z i32;
        byte1 i8;
        byte2 i8; 
    }

    EntityEffect {
        eid i32;
        effect_id EntityEffectKind;
        amplifier i8;
        duration i16;
    }
    RemoveEntityEffect {
        eid i32;
        effect_id EntityEffectKind;
    }
    PickupSpawn {
        eid i32;
        item Slot;
        x AbsoluteInt;
        y AbsoluteInt;
        z AbsoluteInt;
        rotation i8;
        pitch i8;
        roll i8;
    }
    CollectItem {
        collected_eid i32;
        collector_eid i32;
    }
    EntityVelocity {
        eid i32;
        velocity_x i16;
        velocity_y i16;
        velocity_z i16;
    }
    AddObjectVehicle {
        eid i32;
        object_type ObjectVehicleKind;
        x AbsoluteInt;
        y AbsoluteInt;
        z AbsoluteInt;
        fbeid i32; // TODO
    }
    OpenWindow {
        window_id i8;
        inventory_type WindowKind;
        window_title String16;
        num_slots i8;
    }
    CloseWindow {
        wid i8;
    }
    MobSpawn {
        eid i32;
        mobtype EnumMobType;
        x AbsoluteInt;
        y AbsoluteInt;
        z AbsoluteInt;
        yaw RotationFraction360;
        pitch RotationFraction360;
        meta Metadata;
    }
}