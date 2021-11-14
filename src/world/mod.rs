use std::{collections::HashMap, mem::replace, sync::Arc};

use hecs::Entity;
use nbt::decode::read_compound_tag;

use crate::{ecs::{Ecs, entities::player::Player}, game::{BlockPosition, ChunkCoords}};

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
    pub fn process_chunk_loads(&mut self, ecs: &mut Ecs) {
        let chunks = replace(&mut self.load_manager.chunks, HashMap::new());
        for (chunk, data) in chunks.into_iter() {
            if !data.load {
                self.chunks.remove(&chunk);
                for p in ecs.query::<&Player>().iter() {
                    // TODO notify the player of a chunk unloading
                }
            } else {
                self.internal_load_chunk(&chunk);
                for p in ecs.query::<&Player>().iter() {
                    // TODO notify the player of a chunk loading
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
    pub fn unload_chunk(&mut self, coords: &ChunkCoords) {
        self.chunks.remove(coords);
    }
    pub fn from_file_mcr(dir: &str) -> anyhow::Result<Self> {
        let mut file = std::fs::File::open(&format!("{}/level.dat", dir))?;
        let tag = read_compound_tag(&mut file)?;
        let tag = tag
            .get_compound_tag("Data")
            .or(Err(anyhow::anyhow!("Tag read error")))?
            .clone();
        let world = Self {
            generator: Arc::new(Box::new(MountainWorldGenerator::new(
                tag.get_i64("RandomSeed")
                    .or(Err(anyhow::anyhow!("Tag read error")))? as u64,
            ))),
            region_provider: MCRegionLoader::new(dir)?,
            load_manager: ChunkLoadManager::default(),
            chunks: HashMap::new(),
        };
        drop(tag);
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