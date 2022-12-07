// feather license in FEATHER_LICENSE.md
use std::{collections::HashMap, sync::Arc, backtrace};

use ahash::AHashMap;
use parking_lot::{RwLockWriteGuard, RwLockReadGuard};

use crate::{game::{BlockPosition, ChunkCoords}, ecs::systems::world::light::{LightPropagationManager, LightPropagationRequest}, item::item::ItemRegistry};

use super::{chunk_lock::{ChunkHandle, ChunkLock}, BlockState, chunks::Chunk};
pub const CHUNK_HEIGHT: i32 = 128;
pub type ChunkMapInner = AHashMap<ChunkCoords, ChunkHandle>;

/// This struct stores all the chunks on the server,
/// so it allows access to blocks and lighting data.
///
/// Chunks are internally wrapped in `Arc<RwLock>`,
/// allowing multiple systems to access different parts
/// of the world in parallel. Mutable access to this
/// type is only required for inserting and removing
/// chunks.
#[derive(Default)]
pub struct ChunkMap(pub ChunkMapInner);

impl ChunkMap {
    /// Creates a new, empty world.
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieves a handle to the chunk at the given
    /// position, or `None` if it is not loaded.
    pub fn chunk_at(&self, pos: ChunkCoords) -> Option<RwLockReadGuard<Chunk>> {
        self.0.get(&pos).map(|lock| lock.read())
    }

    /// Retrieves a handle to the chunk at the given
    /// position, or `None` if it is not loaded.
    pub fn chunk_at_mut(&self, pos: ChunkCoords) -> Option<RwLockWriteGuard<Chunk>> {
        self.0.get(&pos).map(|lock| lock.write()).flatten()
    }

    /// Returns an `Arc<RwLock<Chunk>>` at the given position.
    pub fn chunk_handle_at(&self, pos: ChunkCoords) -> Option<ChunkHandle> {
        self.0.get(&pos).map(Arc::clone)
    }

    pub fn block_at(&self, pos: BlockPosition) -> Option<BlockState> {
        check_coords(pos)?;

        let (x, y, z) = chunk_relative_pos(pos.into());
        self.chunk_at(pos.to_chunk_coords())
            .map(|chunk| chunk.block_at(x, y, z))
            .flatten()
    }

    pub fn set_block_at(&self, world: i32, mut light: Option<&mut LightPropagationManager>, pos: BlockPosition, block: BlockState, nlh: bool) -> bool {
        if check_coords(pos).is_none() {
            return false;
        }
        let (x, y, z) = chunk_relative_pos(pos.into());
        if let Some(mut chunk) = self.chunk_at_mut(pos.to_chunk_coords()) {
            let original_block = chunk.block_at(x, y, z);
            if nlh {
                chunk.set_block_at_nlh(x, y, z, block);
            } else {
                chunk.set_block_at(x, y, z, block);
            }
            if !nlh {
                light.as_mut().unwrap().push(LightPropagationRequest::ChunkSky { position: chunk.position(), world });

                let l_emittance = original_block.and_then(|v| ItemRegistry::global().get_block(v.b_type)).map(|v| v.light_emittance()).unwrap_or(0);
                light.as_mut().unwrap().push(LightPropagationRequest::BlockLight { position: pos, was_source: l_emittance });
                // for request in chunk.global_skylight_requests() {
                //     light.as_mut().unwrap().push(request);
                // }
            }
            return true;
        }
        false
    }

    /// Returns an iterator over chunks.
    pub fn iter_chunks(&self) -> impl IntoIterator<Item = &ChunkHandle> {
        self.0.values()
    }

    /// Inserts a new chunk into the chunk map.
    pub fn insert_chunk(&mut self, chunk: Chunk) {
        self.0
            .insert(chunk.position(), Arc::new(ChunkLock::new(chunk, true)));
    }

    /// Removes the chunk at the given position, returning `true` if it existed.
    pub fn remove_chunk(&mut self, pos: ChunkCoords) -> bool {
        self.0.remove(&pos).is_some()
    }
}

pub fn check_coords(pos: BlockPosition) -> Option<()> {
    if pos.y >= 0 && pos.y < CHUNK_HEIGHT as i32 {
        Some(())
    } else {
        None
    }
}

pub fn chunk_relative_pos(block_pos: BlockPosition) -> (usize, usize, usize) {
    (
        block_pos.x as usize & 0xf,
        block_pos.y as usize,
        block_pos.z as usize & 0xf,
    )
}
