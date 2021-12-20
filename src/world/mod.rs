pub mod view;
use std::{
    collections::{HashMap, HashSet},
    mem::{replace, take},
    sync::Arc,
};

use hecs::Entity;
use nbt::decode::read_compound_tag;

use crate::{
    ecs::{
        entities::player::{ChunkLoadQueue, Player},
        systems::SysResult,
        Ecs,
    },
    events::ChunkLoadEvent,
    game::{BlockPosition, ChunkCoords, Game, Position},
};

pub mod chunk_lock;
pub mod chunk_subscriptions;
pub mod chunks;
pub mod mcregion;
pub mod chunk_map;
pub mod chunk_entities;
use chunks::*;
use mcregion::*;
use self::{chunk_lock::{ChunkHandle, ChunkLock}, chunk_map::ChunkMap};
pub struct World {
    pub chunk_map: ChunkMap,
    pub region_provider: MCRegionLoader,
    pub loading_chunks: HashSet<ChunkCoords>,
    canceled_chunk_loads: HashSet<ChunkCoords>,
}
impl World {
    pub fn load_chunks(&mut self, ecs: &mut Ecs) -> SysResult {
        let load_queue = take(&mut self.loading_chunks);
        for pos in load_queue {
            //log::info!("Trying to load {:?}", pos);
            let chunk = match self.region_provider.get_chunk(&pos) {
                Some(c) => c,
                None => {
                    log::error!("Can't load chunk {}, It does not exist!", pos);
                    continue;
                }
            }
            .clone();
            self.chunk_map.insert_chunk(chunk);
            ecs.insert_event(ChunkLoadEvent {
                chunk: self.chunk_map.chunk_handle_at(pos).unwrap(),
                position: pos.clone(),
            });
        }
        Ok(())
    }
    /// Unloads the given chunk.
    pub fn unload_chunk(&mut self, pos: &ChunkCoords) -> anyhow::Result<()> {
        if let Some((pos, handle)) = self.chunk_map.0.remove_entry(&pos) {
            handle.set_unloaded()?;
        }
        self.chunk_map.remove_chunk(*pos);
        if self.is_chunk_loading(pos) {
            self.canceled_chunk_loads.insert(*pos);
        }

        log::trace!("Unloaded chunk {:?}", pos);
        Ok(())
    }
    /// Returns whether the given chunk is queued to be loaded.
    pub fn is_chunk_loading(&self, pos: &ChunkCoords) -> bool {
        self.loading_chunks.contains(&pos)
    }
    pub fn queue_chunk_load(&mut self, pos: &ChunkCoords) {
        self.loading_chunks.insert(pos.clone());
    }
    pub fn from_file_mcr(dir: &str) -> anyhow::Result<Self> {
        let mut file = std::fs::File::open(&format!("{}/level.dat", dir))?;
        let world = Self {
            region_provider: MCRegionLoader::new(dir)?,
            chunk_map: ChunkMap::new(),
            loading_chunks: HashSet::new(),
            canceled_chunk_loads: HashSet::new(),
        };
        Ok(world)
    }
    pub fn pos_to_index(x: i32, y: i32, z: i32) -> Option<(i32, i32, i32)> {
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;
        let section = y / 16;
        if section < 0 {
            return None;
        }
        Some((chunk_x, chunk_z, section))
    }
    pub fn is_chunk_loaded(&self, pos: &ChunkCoords) -> bool {
        self.chunk_map.0.contains_key(pos)
    }
}
pub struct ChunkLoadData {
    load: bool,
}
impl ChunkLoadData {
    pub fn new(load: bool) -> Self {
        Self { load }
    }
}
#[derive(Default)]
pub struct ChunkLoadManager {
    chunks: HashMap<ChunkCoords, ChunkLoadData>,
}
impl ChunkLoadManager {
    pub fn load_chunk(&mut self, coords: &ChunkCoords) {
        if let Some(v) = self.chunks.get_mut(coords) {
            v.load = true;
        } else {
            self.chunks.insert(*coords, ChunkLoadData::new(true));
        }
    }
    pub fn unload_chunk(&mut self, coords: &ChunkCoords) {
        if let Some(v) = self.chunks.get_mut(coords) {
            v.load = false;
        } else {
            self.chunks.insert(*coords, ChunkLoadData::new(false));
        }
    }
}
