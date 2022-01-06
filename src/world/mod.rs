pub mod cache;
pub mod generation;
pub mod view;
use std::{
    collections::{HashMap, HashSet},
    mem::{replace, take},
    sync::{atomic::AtomicBool, Arc},
    time::Instant, path::PathBuf, fs::File,
};
pub mod worker;
use ahash::AHashMap;
use hecs::Entity;
use nbt::{decode::read_compound_tag, CompoundTag, encode::write_compound_tag};
use retain_mut::RetainMut;

use crate::{
    aabb::{AABBSize, AABB, SweeptestOutput},
    block_entity::{BlockEntity, BlockEntityNBTLoaders, BlockEntitySaver},
    ecs::{
        entities::player::{ChunkLoadQueue, Player},
        systems::{world::light::LightPropagationManager, SysResult},
        Ecs,
    },
    events::ChunkLoadEvent,
    game::{BlockPosition, ChunkCoords, Game, Position},
    item::item::{block::AtomicRegistryBlock, ItemRegistry},
    world::{worker::SaveRequest, generation::WorldgenRegistry}, configuration::CONFIGURATION, protocol::packets::Face,
};

pub mod chunk_entities;
pub mod chunk_lock;
pub mod chunk_map;
pub mod chunk_subscriptions;
pub mod chunks;
pub mod heightmap;
pub mod light;
pub mod mcregion;
pub mod packed_array;
use self::{
    cache::ChunkCache,
    chunk_lock::{ChunkHandle, ChunkLock},
    chunk_map::ChunkMap,
    generation::{EmptyWorldGenerator, FlatWorldGenerator, TerrainWorldGenerator},
    worker::{ChunkWorker, LoadRequest},
};
use chunks::*;
use mcregion::*;
pub struct World {
    pub id: i32,
    pub chunk_map: ChunkMap,
    pub cache: ChunkCache,
    chunk_worker: ChunkWorker,
    pub loading_chunks: HashSet<ChunkCoords>,
    canceled_chunk_loads: HashSet<ChunkCoords>,
    shutdown: Arc<AtomicBool>,
    pub level_dat: LevelDat,
    pub world_dir: PathBuf,
}
impl World {
    pub fn collides_with(
        &self,
        aabb: &AABBSize,
        position: &Position,
        predicate: AtomicRegistryBlock,
    ) -> bool {
        for (check, _, _) in self.get_collisions(aabb, position) {
            if check == predicate {
                return true;
            }
        }
        false
    }
    pub fn get_collisions_extra(
        &self,
        aabb: &AABBSize,
        position: &Position,
    ) -> Vec<(AtomicRegistryBlock, BlockState, BlockPosition, Vec<Face>)> {
        let mut blocks = Vec::new();
        //log::info!("Called get col");
        let mut registry = ItemRegistry::global();
        for x in (position.x - 3.0 + aabb.minx).floor() as i32
            ..(position.x + 3.0 + aabb.maxx).floor() as i32
        {
            for y in (position.y - 3.0 + aabb.miny).floor() as i32
                ..(position.y + 3.0 + aabb.maxy).floor() as i32
            {
                for z in (position.z - 3.0 + aabb.minz).floor() as i32
                    ..(position.z + 3.0 + aabb.maxz).floor() as i32
                {
                    let p = BlockPosition::new(x, y, z, position.world);
                    if let Some(state) = self.block_at(p) {
                        if !state.is_air() {
                            if let Some(block) = registry.get_block(state.b_type) {
                                blocks.push((block, state, p, Vec::new()));
                            }
                        }
                    }
                }
            }
        }
        let aabb = aabb.get(position);
        blocks.retain_mut(|(block, state, pos, faces)| {
            if let Some(bounding_box) = block.collision_box(*state, *pos) {
                let mut collisions = bounding_box.collisions(&aabb);
                if collisions.len() > 0 {
                    faces.append(&mut collisions);
                    return true;
                }
            }
            false
        });
        blocks
    }
    pub fn get_possible_collisions(&self, aabb: &AABBSize, position: &Position) -> Vec<(AtomicRegistryBlock, BlockState, BlockPosition)> {
        let mut blocks = Vec::new();
        //log::info!("Called get col");
        let registry = ItemRegistry::global();
        for x in (position.x - 3.0 + aabb.minx).floor() as i32
            ..(position.x + 3.0 + aabb.maxx).floor() as i32
        {
            for y in (position.y - 3.0 + aabb.miny).floor() as i32
                ..(position.y + 3.0 + aabb.maxy).floor() as i32
            {
                for z in (position.z - 3.0 + aabb.minz).floor() as i32
                    ..(position.z + 3.0 + aabb.maxz).floor() as i32
                {
                    let p = BlockPosition::new(x, y, z, position.world);
                    if let Some(state) = self.block_at(p) {
                        if !state.is_air() {
                            if let Some(block) = registry.get_block(state.b_type) {
                                blocks.push((block, state, p));
                            }
                        }
                    }
                }
            }
        }
        blocks
    }
    pub fn get_collisions(
        &self,
        aabb: &AABBSize,
        position: &Position,
    ) -> Vec<(AtomicRegistryBlock, BlockState, BlockPosition)> {
        let mut blocks = Vec::new();
        //log::info!("Called get col");
        let mut registry = ItemRegistry::global();
        for x in (position.x - 3.0 + aabb.minx).floor() as i32
            ..(position.x + 3.0 + aabb.maxx).floor() as i32
        {
            for y in (position.y - 3.0 + aabb.miny).floor() as i32
                ..(position.y + 3.0 + aabb.maxy).floor() as i32
            {
                for z in (position.z - 3.0 + aabb.minz).floor() as i32
                    ..(position.z + 3.0 + aabb.maxz).floor() as i32
                {
                    let p = BlockPosition::new(x, y, z, position.world);
                    if let Some(state) = self.block_at(p) {
                        if let Some(block) = registry.get_block(state.b_type) {
                            blocks.push((block, state, p));
                        }
                    }
                }
            }
        }
        let aabb = aabb.get(position);
        blocks.retain(|(block, state, pos)| {
            if let Some(bounding_box) = block.collision_box(*state, *pos) {
                if bounding_box.intersects(&aabb) {
                    return true;
                }
            }
            false
        });
        blocks
    }
    pub fn drop_chunk_sender(&mut self) {
        self.chunk_worker.drop_sender();
    }
    pub fn get_shutdown(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.shutdown)
    }
    /// Retrieves the block at the specified
    /// location. If the chunk in which the block
    /// exists is not loaded or the coordinates
    /// are out of bounds, `None` is returned.
    pub fn block_at(&self, pos: BlockPosition) -> Option<BlockState> {
        self.chunk_map.block_at(pos)
    }
    /// Sets the block at the given position.
    ///
    /// Returns `true` if the block was set, or `false`
    /// if its chunk was not loaded or the coordinates
    /// are out of bounds and thus no operation
    /// was performed.
    pub fn set_block_at(
        &self,
        light: &mut LightPropagationManager,
        pos: BlockPosition,
        block: BlockState,
        nlh: bool,
    ) -> bool {
        if pos.within_border(CONFIGURATION.world_border) {
            return self.chunk_map.set_block_at(self.id, light, pos, block, nlh);
        }
        false
    }
    pub fn load_chunks(
        &mut self,
        ecs: &mut Ecs,
        light: &mut LightPropagationManager,
    ) -> anyhow::Result<Vec<CompoundTag>> {
        let mut tes = Vec::new();
        while let Some(mut loaded) = self.chunk_worker.poll_loaded_chunk(light)? {
            self.loading_chunks.remove(&loaded.pos);
            if self.canceled_chunk_loads.remove(&loaded.pos) {
                continue;
            }
            let chunk = loaded.chunk;
            self.chunk_map.insert_chunk(chunk);
            tes.append(&mut loaded.tile_entity_data);
            ecs.insert_event(ChunkLoadEvent {
                chunk: Arc::clone(&self.chunk_map.0[&loaded.pos]),
                position: loaded.pos,
            });
        }
        Ok(tes)
    }
    /// Unloads the given chunk.
    pub fn unload_chunk(&mut self, ecs: &mut Ecs, pos: &ChunkCoords) -> anyhow::Result<()> {
        if let Some((pos, handle)) = self.chunk_map.0.remove_entry(&pos) {
            if CONFIGURATION.logging.chunk_unload {
                log::info!("Unloading chunk at {}", pos);
            }
            handle.set_unloaded()?;
            let mut block_entity_data = Vec::new();
            for (entity, (saver, be)) in ecs.query::<(&BlockEntitySaver, &BlockEntity)>().iter() {
                //log::info!("Pos: {} {}", be.0.to_chunk_coords(), pos);
                if be.0.to_chunk_coords() == pos {
                    block_entity_data.push(saver.save(
                        &ecs.entity(entity)?,
                        &saver.be_type,
                        be.0,
                    )?);
                }
            }
            self.chunk_worker.queue_chunk_save(SaveRequest {
                pos,
                chunk: handle.clone(),
                block_entities: block_entity_data,
            });
            self.cache.insert(pos, handle);
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
    pub fn queue_chunk_load(&mut self, req: LoadRequest) {
        let pos = req.pos;
        if self.cache.contains(&pos) {
            self.chunk_map
                .0
                .insert(pos, self.cache.remove(pos).unwrap());
            self.chunk_map.chunk_handle_at(pos).unwrap().set_loaded();
        } else {
            self.loading_chunks.insert(req.pos);
            self.chunk_worker.queue_load(req);
        }
    }
    pub fn from_file_mcr(dir: &str, world_type: i8) -> anyhow::Result<Self> {
        log::info!("Loading world from {:?}", dir);
        let dir = PathBuf::from(dir);
        let mut level_dat = dir.clone();
        level_dat.push("level.dat");
        let level_dat = LevelDat::from_file(level_dat, world_type as i32);
        let shutdown = Arc::new(AtomicBool::new(false));
        let generator = WorldgenRegistry::get().get_generator(&CONFIGURATION.chunk_generator.name(), level_dat.world_seed, world_type).ok_or(anyhow::anyhow!("No such generator"))?;
        let world = Self {
            chunk_worker: ChunkWorker::new(dir.clone(), level_dat.world_seed, generator, shutdown.clone()),
            chunk_map: ChunkMap::new(),
            loading_chunks: HashSet::new(),
            canceled_chunk_loads: HashSet::new(),
            shutdown,
            cache: ChunkCache::new(),
            id: world_type as i32,
            level_dat,
            world_dir: dir
        };
        log::info!("Using world generator {:?}", CONFIGURATION.chunk_generator.name());
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
    pub fn loaded_chunks(&self) -> Vec<ChunkCoords> {
        self.chunk_map.0.keys().cloned().collect()
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
    chunks: AHashMap<ChunkCoords, ChunkLoadData>,
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


pub struct LevelDat {
    pub spawn_point: BlockPosition,
    pub world_seed: u64,
}
impl LevelDat {
    pub fn from_file(input: impl Into<PathBuf>, world_type: i32) -> Self {
        if let Ok(mut input) = File::open(input.into()) {
            let mut x: Box<dyn FnMut() -> Option<Self>> = Box::new(|| {
                let tag = read_compound_tag(&mut input).ok()?;
                let tag = tag.get_compound_tag("Data").ok()?;
                let spawn_x = tag.get_i32("SpawnX").unwrap_or(0);
                let spawn_y = tag.get_i32("SpawnY").unwrap_or(75);
                let spawn_z = tag.get_i32("SpawnZ").unwrap_or(0);
                let seed = tag.get_i64("RandomSeed").unwrap_or(CONFIGURATION.world_seed.seed as i64) as u64;
                Some(Self { spawn_point: BlockPosition::new(spawn_x, spawn_y, spawn_z, world_type), world_seed: seed })
            });
            let x = x();
            if let Some(x) = x {
                x
            } else {
                Self { spawn_point: BlockPosition::new(0, 75, 0, world_type), world_seed: CONFIGURATION.world_seed.seed as u64 }
            }
        } else {
            Self { spawn_point: BlockPosition::new(0, 75, 0, world_type), world_seed: CONFIGURATION.world_seed.seed as u64 }
        }
    }
    pub fn to_file(&self, file: impl Into<PathBuf>) -> anyhow::Result<()> {
        let file = file.into();
        let mut file = File::create(file)?;
        let mut tag = CompoundTag::new();
        let mut data = CompoundTag::new();
        data.insert_i32("SpawnX", self.spawn_point.x);
        data.insert_i32("SpawnY", self.spawn_point.y);
        data.insert_i32("SpawnZ", self.spawn_point.z);
        data.insert_i64("RandomSeed", self.world_seed as i64);
        tag.insert("Data", data);
        write_compound_tag(&mut file, &tag)?;
        Ok(())
    }
}