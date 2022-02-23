use std::{collections::VecDeque, ops::Deref, sync::Arc};

use ahash::AHashMap;
use flume::{Receiver, Sender};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use rustc_hash::FxHashMap;

use crate::{
    ecs::systems::world::light::LightPropagationRequest,
    game::{BlockPosition, ChunkCoords, Game},
    item::item::ItemRegistry,
    protocol::packets::Face, world::chunks::CHUNK_WIDTH,
};

use super::{
    cache::ChunkCache,
    chunk_lock::ChunkHandle,
    chunk_map::{check_coords, chunk_relative_pos, ChunkMap, CHUNK_HEIGHT},
    chunks::{BlockState, Chunk},
};
pub enum GameCommand {
    GetChunk {
        position: ChunkCoords,
        recv: oneshot::Sender<Option<ChunkHandle>>,
    },
    GetBlock {
        position: BlockPosition,
        recv: oneshot::Sender<Option<BlockState>>,
    },
    SetBlockLight {
        position: BlockPosition,
        state: BlockState,
        recv: oneshot::Sender<()>,
    },
}
pub struct LightThreadManager {
    sender: Sender<GameCommand>,
    receiver: Receiver<LightPropagationRequest>,
}
impl LightThreadManager {
    pub fn new() -> (Self, LightThreadSync) {
        let (send, recv) = flume::unbounded();
        let (send_light, recv_light) = flume::unbounded();
        (
            Self {
                sender: send,
                receiver: recv_light,
            },
            LightThreadSync {
                receiver: recv,
                sender: send_light,
            },
        )
    }
    pub fn block(&mut self, position: BlockPosition) -> Option<BlockState> {
        let (send, recv) = oneshot::channel();
        let command = GameCommand::GetBlock {
            position,
            recv: send,
        };
        self.sender.send(command).expect("handle later");
        recv.recv().unwrap()
    }
    pub fn chunk(&mut self, position: ChunkCoords) -> Option<ChunkHandle> {
        let (send, recv) = oneshot::channel();
        let command = GameCommand::GetChunk {
            position,
            recv: send,
        };
        self.sender.send(command).expect("handle later");
        recv.recv().unwrap()
    }
    pub fn set_block(&mut self, position: BlockPosition, state: BlockState) {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(GameCommand::SetBlockLight {
                position,
                state,
                recv: send,
            })
            .expect("handle later");
        recv.recv().unwrap()
    }
    pub fn run(mut self) {
        let mut propagator = LightPropagator::new(true); // TODO: un harrdcode
        let mut num = 0;
        loop {
            let values = self
                .receiver
                .drain()
                .collect::<Vec<LightPropagationRequest>>();
            for v in values {
                match v {
                    LightPropagationRequest::ChunkSky { position, world } => {
                        //log::info!("\n\n\nNEW #{}\n\n\n", num);
                        let mut maps = None;
                        for _ in 0..5 {
                            propagator.map.add_to_regmap(position, &mut self);
                            if let Some(ch) = propagator.map.1.chunk_at(position) {
                                maps = Some(ch.heightmaps.clone());
                                drop(ch);
                                break;
                            }
                        }
                        let mut posses = Vec::new();
                        propagator.map.lock(position, &mut self);
                        for x in 0..CHUNK_WIDTH as i32 {
                            for z in 0..CHUNK_WIDTH as i32 {
                                for y in 0..CHUNK_HEIGHT as i32 {
                                    let pos = BlockPosition::new((position.x * 16) + x, y, (position.z * 16) + z, world);
                                    if let Some(mut b) = propagator.map.block_at(pos, &mut self) {
                                        b.b_skylight = 0;
                                        propagator.map.set_block_at(pos, b, &mut self);
                                    }
                                }
                            }
                        }
                        if let Some(maps) = maps {
                            //log::info!("Some maps");
                            for x in 0..CHUNK_WIDTH as i32 {
                                for z in 0..CHUNK_WIDTH as i32 {
                                    let height = maps.light_blocking.height(x as usize, z as usize).map(|v| v as i32).unwrap_or(CHUNK_HEIGHT);
                                    for y in (height..CHUNK_HEIGHT).rev() {
                                        let pos = BlockPosition::new((position.x * 16) + x, y, (position.z * 16) + z, world);
                                        posses.push(pos);
                                    }
                                }
                            }
                        } else {
                            //log::info!("Nun maps");
                        }
                        propagator.map.free(position);
                        for pos in posses {
                            //log::info!("Doing position {:?}", pos);
                            propagator.increase_light(&mut self, pos, 15);
                        }
                    }
                }
            }
        }
        // while let Ok(v) = self.receiver.recv() {
        //     log::info!("\n\n\nNEW #{}\n\n\n", num);
        //     num += 1;
        //     propagator.increase_light(&mut self, v.position, v.level);
        // }
    }
}

pub struct LockedChunk {
    handle: ChunkHandle,
    data: *mut Chunk,
}
impl Drop for LockedChunk {
    fn drop(&mut self) {
        unsafe {
            self.handle.lock.force_unlock_write_fair();
        }
    }
}
impl LockedChunk {
    pub fn new(handle: ChunkHandle) -> Option<Self> {
        let v = handle.lock.write();
        std::mem::forget(v);
        Some(Self {
            handle: handle.clone(),
            data: handle.lock.data_ptr(),
        })
    }
    pub fn get(&mut self) -> &mut Chunk {
        unsafe { self.data.as_mut().unwrap() }
    }
}
#[derive(Default)]
pub struct RegChunkMap(pub FxHashMap<ChunkCoords, ChunkHandle>);

impl RegChunkMap {
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

    pub fn set_block_at(&self, pos: BlockPosition, block: BlockState, nlh: bool) -> bool {
        if check_coords(pos).is_none() {
            return false;
        }
        let (x, y, z) = chunk_relative_pos(pos.into());
        if let Some(mut chunk) = self.chunk_at_mut(pos.to_chunk_coords()) {
            chunk.set_block_at_nlh(x, y, z, block);
            return true;
        }
        false
    }

    /// Returns an iterator over chunks.
    pub fn iter_chunks(&self) -> impl IntoIterator<Item = &ChunkHandle> {
        self.0.values()
    }

    /// Inserts a new chunk into the chunk map.
    pub fn insert_chunk(&mut self, chunk: ChunkHandle) {
        let p = chunk.read().position();
        self.0
            .insert(p, chunk);
    }

    /// Removes the chunk at the given position, returning `true` if it existed.
    pub fn remove_chunk(&mut self, pos: ChunkCoords) -> bool {
        self.0.remove(&pos).is_some()
    }

    pub fn has_chunk(&self, pos: ChunkCoords) -> bool {
        self.0.contains_key(&pos)
    }
}

#[derive(Default)]
pub struct LockedChunkMap(pub FxHashMap<ChunkCoords, LockedChunk>, pub RegChunkMap, pub FxHashMap<ChunkCoords, usize>);

impl LockedChunkMap {
    /// Creates a new, empty world.
    pub fn new() -> Self {
        Self::default()
    }
    /// Retrieves a handle to the chunk at the given
    /// position, or `None` if it is not loaded.
    fn chunk_at(
        &mut self,
        pos: ChunkCoords,
        manager: &mut LightThreadManager,
    ) -> Option<&mut LockedChunk> {
        if self.0.contains_key(&pos) {
            Some(self.0.get_mut(&pos).unwrap())
        } else {
            None
        }
    }
    pub fn add_to_regmap(&mut self, pos: ChunkCoords, manager: &mut LightThreadManager) {
        if !self.1.has_chunk(pos) {
            let count = self.2.get(&pos).cloned().unwrap_or(0);
            if count < 3 { // more than 3 failed responses, accept that it does not exist
                if let Some(v) = manager.chunk(pos) {
                    self.1.insert_chunk(v);
                } else if let Some(v) = self.2.get_mut(&pos) {
                    *v += 1;
                } else {
                    self.2.insert(pos, 1);
                }
            }
        }
    }
    pub fn lock(&mut self, pos: ChunkCoords, manager: &mut LightThreadManager) {
        self.add_to_regmap(pos, manager);
        if let Some(v) = self.1.chunk_handle_at(pos) {
            self.insert_locked_chunk(v);
        }
    }
    pub fn free(&mut self, pos: ChunkCoords) {
        self.0.remove(&pos);
    }
    pub fn block_at(
        &mut self,
        pos: BlockPosition,
        manager: &mut LightThreadManager,
    ) -> Option<BlockState> {
        if self.0.contains_key(&pos.to_chunk_coords()) {
            check_coords(pos)?;

            let (x, y, z) = chunk_relative_pos(pos.into());
            self.chunk_at(pos.to_chunk_coords(), manager)
                .map(|chunk| chunk.get().block_at(x, y, z))
                .flatten()
        } else {
            self.add_to_regmap(pos.to_chunk_coords(), manager);
            self.1.block_at(pos)
        }
    }

    pub fn set_block_at(
        &mut self,
        pos: BlockPosition,
        block: BlockState,
        manager: &mut LightThreadManager,
    ) -> bool {
        if self.0.contains_key(&pos.to_chunk_coords()) {
            if check_coords(pos).is_none() {
                return false;
            }
            let (x, y, z) = chunk_relative_pos(pos.into());
            if let Some(mut chunk) = self.chunk_at(pos.to_chunk_coords(), manager) {
                chunk.get().set_block_at_nlh(x, y, z, block);
                return true;
            }
            false
        } else {
            self.add_to_regmap(pos.to_chunk_coords(), manager);
            self.1.set_block_at(pos, block, true)
        }
    }
    /// Inserts a new chunk into the chunk map.
    pub fn insert_locked_chunk(&mut self, handle: ChunkHandle) {
        let mut chunk = LockedChunk::new(handle).unwrap();
        self.0.insert(chunk.get().position(), chunk);
    }

    /// Removes the chunk at the given position, returning `true` if it existed.
    pub fn remove_chunk(&mut self, pos: ChunkCoords) -> bool {
        self.0.remove(&pos).is_some()
    }
}

pub struct LightThreadSync {
    pub receiver: Receiver<GameCommand>,
    pub sender: Sender<LightPropagationRequest>,
}
pub struct LightPropagator {
    sky_light: bool,
    queue: VecDeque<(BlockPosition, u8)>,
    map: LockedChunkMap,
}
impl LightPropagator {
    pub fn clear_map(&mut self) {
        self.map = LockedChunkMap::new();
    }
    pub fn new(sky_light: bool) -> Self {
        Self {
            sky_light,
            queue: VecDeque::new(),
            map: LockedChunkMap::new(),
        }
    }
    pub fn increase_light(
        &mut self,
        game: &mut LightThreadManager,
        position: BlockPosition,
        value: u8,
    ) {
        if value > 15 {
            return;
        }
        if let Some(mut state) = self.map.block_at(position, game) {
            let light = match self.sky_light {
                true => state.b_skylight,
                false => state.b_light,
            };
            if light < value || self.sky_light {
                self.queue.push_back((position, value));
                match self.sky_light {
                    true => state.b_skylight = value,
                    false => state.b_light = value,
                }
                self.map.set_block_at(position, state, game);
                self.propagate(game);
            }
        }
    }
    fn propagate(&mut self, game: &mut LightThreadManager) {
        let mut registry = ItemRegistry::global();
        while !self.queue.is_empty() {
            //log::info!("In queue");
            let (pos, light_value) = self.queue.pop_front().unwrap();
            for face in Face::all_faces() {
                let neighbor = face.offset(pos);
                self.map.lock(neighbor.to_chunk_coords(), game);
                if let Some(neighbor_state) = self.map.block_at(neighbor, game) {
                    let current_level;
                    current_level = match self.sky_light {
                        true => neighbor_state.b_skylight,
                        false => neighbor_state.b_light,
                    };
                    //log::info!("Neighbor on {:?} is {}", face, current_level);
                    if current_level >= (light_value - 1) {
                        self.map.free(neighbor.to_chunk_coords());
                        continue;
                    }
                    let mut target_level =
                        light_value - 1.max(Self::opacity(neighbor_state.b_type, registry.clone()));
                    if target_level > 15 {
                        target_level = 0;
                    }
                    if target_level > current_level {
                        if let Some(mut bstate) = self.map.block_at(neighbor, game) {
                            match self.sky_light {
                                true => bstate.b_skylight = target_level,
                                false => bstate.b_light = target_level,
                            }
                            self.map.set_block_at(neighbor, bstate, game);
                        }
                        self.queue.push_back((neighbor, target_level));
                    }
                }
                self.map.free(neighbor.to_chunk_coords());
            }
        }
    }
    fn opacity(id: u8, registry: Arc<ItemRegistry>) -> u8 {
        if let Some(block) = registry.get_block(id) {
            return block.opacity();
        }
        15
    }
}

pub fn propagate_light(
    world: i32,
    game: &mut Game,
    position: BlockPosition,
    mut light_level: u8,
    sky_light: bool,
) {
    //log::info!("Called");
    if game.is_solid_block(position, world) {
        log::info!("Solid");
        return;
    }
    if let Some(block) = game.block(position, world) {
        if let Some(block) = ItemRegistry::global().get_block(block.b_type) {
            // //let level = block.light_filter_level();
            // if level > 0 {
            //     log::info!("Level is {}", level);
            //     light_level -= level;
            // }
        }
    }
    light_level -= 1;
    if light_level == 0 {
        return;
    }
    if let Some(mut block) = game.block(position, world) {
        let light = match sky_light {
            true => block.b_skylight,
            false => block.b_light,
        };
        if light_level == 0 {
            log::info!(
                "skylight? {} of {:?} is {}, higher than {}",
                sky_light,
                position,
                light,
                light_level
            );
            return;
        }
        if light >= light_level {
            return;
        }
        match sky_light {
            true => block.b_skylight = light,
            false => block.b_light = light,
        }
        log::info!(
            "Setting skylight? {} of {:?} to {}",
            sky_light,
            position,
            light
        );
        game.set_block(position, block, world);
        for face in Face::all_faces() {
            log::info!(
                "Propagating to {:?} with a light level of {}",
                face,
                light_level
            );
            propagate_light(world, game, face.offset(position), light_level, sky_light);
        }
    } else {
        log::info!("No block");
    }
}
