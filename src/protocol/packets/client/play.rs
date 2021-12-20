use super::*;
use crate::{protocol::io::String16};

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
    PlayerBlockPlacement {
        x i32;
        y i8;
        z i32;
        direction Face;
        block PBPBlockItem;
    }
    CloseWindow {
        window_id i8;
    }
    UseEntity {
        user i32;
        target i32;
        left_click bool;
    }
}

#[derive(Clone, Debug)]
pub struct PBPBlockItem {
    pub block_or_item_id: i16,
    pub amount: Option<i8>,
    pub damage: Option<i16>,
}
impl Readable for PBPBlockItem {
    fn read(buffer: &mut std::io::Cursor<&[u8]>, version: crate::protocol::ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized {
        let block_or_item_id = i16::read(buffer, version)?;
        let mut amount = None;
        let mut damage = None;
        if block_or_item_id >= 0 {
            amount = Some(i8::read(buffer, version)?);
            damage = Some(i16::read(buffer, version)?);
        }
        Ok(Self { block_or_item_id, amount, damage })
    }
}

impl Writeable for PBPBlockItem {
    fn write(&self, buffer: &mut Vec<u8>, version: crate::protocol::ProtocolVersion) -> anyhow::Result<()> {
        self.block_or_item_id.write(buffer, version)?;
        if let Some(v) = self.amount {
            v.write(buffer, version)?;
        }
        if let Some(v) = self.damage {
            v.write(buffer, version)?;
        }
        Ok(())
    }
}
