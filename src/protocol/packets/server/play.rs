use anyhow::bail;

pub use chunk_data::{ChunkData, ChunkDataKind};
use crate::protocol::{Readable, Writeable};

use super::*;

mod chunk_data;
packets! {
    PlayerPositionAndLook {
        x f64;
        stance f64;
        y f64;
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
}