use super::*;
use crate::{protocol::io::{String16, Slot}, game::BlockPosition};

packets! {
    ChatMessage {
        message String16;
    }
    PlayerPositionAndLook {
        x f64;
        y f64;
        stance f64;
        z f64;
        yaw f32;
        pitch f32;
        on_ground bool;
    }
    PlayerLook {
        yaw f32;
        pitch f32;
        on_ground bool;
    }
    PlayerPosition {
        x f64;
        y f64;
        stance f64;
        z f64;
        on_ground bool;
    }
    PlayerMovement {
        on_ground bool;
    }
    KeepAlive {
        id i32;
    }
    Disconnect {
        reason String16;
    }
    EntityAction {
        eid i32;
        action_id EntityActionKind;
    }
    Animation {
        eid i32;
        animation EntityAnimationType;
    }
    PlayerDigging {
        status DiggingStatus;
        x i32;
        y i8;
        z i32;
        face Face;
    }
    CreativeInventoryAction {
        slot_id i16;
        item_id i16;
        quantity i16;
        meta i16;
    }
    HoldingChange {
        slot_id i16;
    }
    CloseWindow {
        window_id i8;
    }
    UseEntity {
        user i32;
        target i32;
        left_click bool;
    }
    WindowClick {
        window_id i8;
        slot i16;
        right_click bool;
        action_number i16;
        shift bool;
        item Slot;
    }
    Transaction {
        window_id i8;
        action_number i16;
        accepted bool;
    }
    Respawn {
        world i8;
        difficulty i8;
        gamemode i8;
        world_height i16;
        map_seed i64;
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
}

#[derive(Clone, Debug)]
pub struct PlayerBlockPlacement {
    pub pos: BlockPosition,
    pub direction: Face,
    pub block_or_item: Slot,
}
impl Readable for PlayerBlockPlacement {
    fn read(buffer: &mut std::io::Cursor<&[u8]>, version: crate::protocol::ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized {
        let pos = BlockPosition::read(buffer, version)?;
        let direction = Face::read(buffer, version)?;
        let slot = Slot::read(buffer, version)?;
        Ok(Self { pos, direction, block_or_item: slot })
    }
}

impl Writeable for PlayerBlockPlacement {
    fn write(&self, buffer: &mut Vec<u8>, version: crate::protocol::ProtocolVersion) -> anyhow::Result<()> {
        todo!();
        Ok(())
    }
}
