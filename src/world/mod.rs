use std::{collections::HashMap, mem::{replace, take}, sync::Arc};

use hecs::Entity;
use nbt::decode::read_compound_tag;

use crate::{ecs::{Ecs, entities::player::{ChunkLoadQueue, NetworkManager, Player}}, game::{BlockPosition, ChunkCoords, Position}, network::packet::ServerPacket};

pub mod chunks;
pub mod mcregion;
use chunks::*;
use mcregion::*;
pub struct World {
    pub chunks: HashMap<ChunkCoords, Chunk>,
    pub region_provider: MCRegionLoader,
    pub load_manager: ChunkLoadManager,
    pub generator: Arc<Box<dyn WorldGenerator>>,
}
impl World {
    pub fn unload_unused_chunks(&mut self, ecs: &mut Ecs) {
        let mut all_loaded = Vec::new();
        for (_, (_, c)) in ecs.query::<(&Player, &ChunkLoadQueue)>().iter() {
            for c in c.chunks.iter() {
                if !all_loaded.contains(c) {
                    all_loaded.push(c.clone());
                }
            }
        }
        let mut to_unload = Vec::new();
        for chunk in self.chunks.iter() {
            if !all_loaded.contains(&chunk.0) {
                to_unload.push(chunk.0.clone());
            } 
        }
        for c in to_unload {
            self.unload_chunk(&c);
        }
    }
    pub fn process_chunk_loads(&mut self, ecs: &mut Ecs) {
        self.unload_unused_chunks(ecs);
        let chunks = take(&mut self.load_manager.chunks);
        //log::info!("Len: {}", chunks.len());
        for (chunk, data) in chunks.into_iter() {
            if !data.load {
                self.chunks.remove(&chunk);
                for (_, (_, n, q)) in ecs.query::<(&Player, &mut NetworkManager, &mut ChunkLoadQueue)>().iter() {
                    if q.contains(&chunk) {
                        n.write(ServerPacket::PreChunk {mode: false, x: chunk.x, z: chunk.z});
                    }
                    q.remove(&chunk);
                }
            } else {
                self.internal_load_chunk(&chunk);
                for (_, (_, n, q)) in ecs.query::<(&Player, &mut NetworkManager, &mut ChunkLoadQueue)>().iter() {
                    if q.contains(&chunk) {
                        if let Some(c) = self.chunks.get(&chunk) {
                            n.write(ServerPacket::PreChunk {mode: true, x: chunk.x, z: chunk.z});
                            c.to_packets(n);
                        } else {
                            log::error!("Expected chunk at ({}, {})", chunk.x, chunk.z);
                        }
                    }
                }
            }
        }
    }
    fn internal_load_chunk(&mut self, coords: &ChunkCoords) {
        if let Some(c) = self.region_provider.get_chunk(coords) {
            self.chunks.insert(c.pos, c);
        } else {
            // TODO async gen
            let c = self.generator.gen_chunk(*coords);
            self.chunks.insert(c.pos, c);
        }
    }
    pub fn load_chunk(&mut self, coords: &ChunkCoords) {
        log::info!("Loading chunk ({}, {})", coords.x, coords.z);
        self.load_manager.load_chunk(coords);
    }
    pub fn unload_chunk(&mut self, coords: &ChunkCoords) {
        log::info!("Unloading chunk ({}, {})", coords.x, coords.z);
        self.load_manager.unload_chunk(coords);
    }
    pub fn from_file_mcr(dir: &str) -> anyhow::Result<Self> {
        let mut file = std::fs::File::open(&format!("{}/level.dat", dir))?;
        let world = Self {
            generator: Arc::new(Box::new(MountainWorldGenerator::new(
                0,
            ))),
            region_provider: MCRegionLoader::new(dir)?,
            load_manager: ChunkLoadManager::default(),
            chunks: HashMap::new(),
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
    pub fn set_block(&mut self, pos: &BlockPosition, b: &Block) -> Option<()> {
        if !self.is_chunk_loaded(&pos.to_chunk_coords()) {
            return None;
        }
        let idx = Self::pos_to_index(pos.x, pos.y, pos.z)?;
        let chunk = self.chunks.get_mut(&pos.to_chunk_coords()).unwrap();
        let section = chunk.data.get_mut(idx.2 as usize)?;
        if section.is_none() {
            *section = Some(ChunkSection::new(idx.0, idx.1, idx.2 as i8));
        }
        let section = section.as_mut().unwrap();
        *section.get_block(ChunkSection::pos_to_index(
            pos.x.rem_euclid(16),
            pos.y.rem_euclid(16),
            pos.z.rem_euclid(16),
        ))? = b.clone();
        Some(())
    }
    pub fn get_block(&mut self, pos: &BlockPosition) -> Option<Block> {
        if !self.is_chunk_loaded(&pos.to_chunk_coords()) {
            return None;
        }
        let idx = Self::pos_to_index(pos.x, pos.y, pos.z).unwrap();
        let chunk = self.chunks.get_mut(&pos.to_chunk_coords()).unwrap();
        let section = chunk.data.get_mut(idx.2 as usize).unwrap();
        if section.is_none() {
            *section = Some(ChunkSection::new(idx.0, idx.1, idx.2 as i8));
        }
        let section = section.as_mut().unwrap();
        section.get_block(ChunkSection::pos_to_index(
            pos.x.rem_euclid(16),
            pos.y.rem_euclid(16),
            pos.z.rem_euclid(16),
        )).cloned()
    }
    pub fn is_chunk_loaded(&self, pos: &ChunkCoords) -> bool {
        self.chunks.contains_key(pos)
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