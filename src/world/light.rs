use std::{collections::VecDeque, ops::Deref, sync::Arc};

use ahash::AHashMap;
use flume::{Receiver, Sender};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    ecs::systems::world::light::LightPropagationRequest,
    game::{BlockPosition, ChunkCoords, Game},
    item::item::ItemRegistry,
    protocol::packets::Face,
    world::chunks::CHUNK_WIDTH,
};

use super::{
    cache::ChunkCache,
    chunk_lock::ChunkHandle,
    chunk_map::{check_coords, chunk_relative_pos, ChunkMap, CHUNK_HEIGHT},
    chunks::{BlockState, Chunk, SECTION_HEIGHT},
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
    self_sender: Sender<LightPropagationRequest>
}
impl LightThreadManager {
    pub fn new() -> (Self, LightThreadSync) {
        let (send, recv) = flume::unbounded();
        let (send_light, recv_light) = flume::unbounded();
        (
            Self {
                sender: send,
                receiver: recv_light,
                self_sender: send_light.clone()
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

    fn blocklight_for(&mut self, propagator: &mut LightPropagator, position: BlockPosition, was_source: u8) {
        let range_start = position.offset(-16, -16, -16);
        let range_end = position.offset(16, 16, 16);

        let start_chunk = range_start.to_chunk_coords();
        let end_chunk = range_end.to_chunk_coords();

        let start_x_sectionspace = range_start.x as usize % CHUNK_WIDTH;
        let end_x_sectionspace = range_end.x as usize % CHUNK_WIDTH;



        let start_y_sectionspace = range_start.y as usize % SECTION_HEIGHT;
        let end_y_sectionspace = range_end.y as usize % SECTION_HEIGHT;



        let start_z_sectionspace = range_start.z as usize % CHUNK_WIDTH;
        let end_z_sectionspace = range_end.z as usize % CHUNK_WIDTH;

        let start_y_section = range_start.y as usize / SECTION_HEIGHT;
        let end_y_section = range_end.y as usize / SECTION_HEIGHT;

        let mut blocks: Vec<BlockPosition> = Vec::new();

        // log::info!("start_x_sectionspace {}", start_x_sectionspace);
        // log::info!("end_x_sectionspace {}", end_x_sectionspace);

        // log::info!("start_y_sectionspace {}", start_y_sectionspace);
        // log::info!("end_y_sectionspace {}", end_y_sectionspace);

        // log::info!("start_z_sectionspace {}", start_z_sectionspace);
        // log::info!("end_z_sectionspace {}", end_z_sectionspace);

        // log::info!("Checking {}, {}, {} to {}, {}, {}", start_chunk.x, start_y_section, start_chunk.z, end_chunk.x, end_y_section, end_chunk.z);
        for x in start_chunk.x..end_chunk.x + 1 {
            for z in start_chunk.z..end_chunk.z + 1 {
                
                let pos = ChunkCoords::new(x, z, position.world);
                //log::info!("Doing chunk {} (starts at ({}, {}))", pos, pos.x << 4, pos.z << 4);
                propagator.map.add_to_regmap(pos, self);
                if let Some(chunk) = propagator.map.chunk_at(pos, self) {
                    for section_number in start_y_section..end_y_section {
                        if let Some(section) = &chunk.data[section_number] {
                            for (ls_x, ls_y, ls_z) in section.lights() {
                                //log::info!("Light at {} {} {}", ls_x, ls_y, ls_z);
                                let real_x = *ls_x as i32 + (pos.x * CHUNK_WIDTH as i32);
                                let real_y = *ls_y as i32 + (section_number as i32 * SECTION_HEIGHT as i32);
                                let real_z = *ls_z as i32 + (pos.z * CHUNK_WIDTH as i32);

                                if real_y >= range_start.y
                                    && real_y <= range_end.y
                                    && real_z >= range_start.z
                                    && real_z <= range_end.z
                                    && real_x >= range_start.x
                                    && real_x <= range_end.x
                                {
                                    blocks.push(BlockPosition::new(real_x as i32, real_y as i32, real_z as i32, position.world));
                                }
                            }
                        }
                    }
                }
            }
        }

        if was_source > 0 {
            //log::info!("Decreasing");
            propagator.decrease_light(self, position, was_source);
        }

        //log::info!("{} light sources nearby: {:?}", blocks.len(), blocks);
        for pos in blocks {
            propagator.increase_light(self, pos, 15, false);
        }
    }

    fn skylight_for(
        &mut self,
        propagator: &mut LightPropagator,
        position: ChunkCoords,
        world: i32,
        already_done: &mut FxHashSet<ChunkCoords>,
        later_queue: &mut Vec<(ChunkCoords, i32)>,
    ) {
        if already_done.contains(&position) {
            return;
        }
        already_done.insert(position);

        let mut maps = None;
        for _ in 0..5 {
            propagator.map.add_to_regmap(position, self);
            if let Some(ch) = propagator.map.1.chunk_at(position) {
                maps = Some(ch.heightmaps.clone());
                drop(ch);
                break;
            }
        }

        if let Some(maps) = maps {
            let mut posses = Vec::new();
            propagator.map.lock(position, self);
            for x in 0..CHUNK_WIDTH as i32 {
                for z in 0..CHUNK_WIDTH as i32 {
                    for y in 0..CHUNK_HEIGHT as i32 {
                        let pos = BlockPosition::new(
                            (position.x * 16) + x,
                            y,
                            (position.z * 16) + z,
                            world,
                        );
                        if let Some(mut b) = propagator.map.block_at(pos, self) {
                            b.b_skylight = 0;
                            propagator.map.set_block_at(pos, b, self);
                        }
                    }
                }
            }

            //log::info!("Some maps");
            for x in 0..CHUNK_WIDTH as i32 {
                for z in 0..CHUNK_WIDTH as i32 {
                    let height = maps
                        .light_blocking
                        .height(x as usize, z as usize)
                        .map(|v| v as i32)
                        .unwrap_or(CHUNK_HEIGHT);
                    for y in (height..CHUNK_HEIGHT).rev() {
                        let pos = BlockPosition::new(
                            (position.x * 16) + x,
                            y,
                            (position.z * 16) + z,
                            world,
                        );
                        posses.push(pos);
                    }
                }
            }
            for pos in posses {
                propagator.increase_light(self, pos, 15, true);
            }
        } else {
            later_queue.push((position, world));
            return;
        }
    }
    pub fn run(mut self) {
        let mut propagator = LightPropagator::new(); // TODO: un harrdcode
        let mut num = 0;

        let mut later_queue = vec![];
        loop {
            let mut already_done = FxHashSet::default();
            for (position, world) in std::mem::take(&mut later_queue) {
                self.skylight_for(
                    &mut propagator,
                    position,
                    world,
                    &mut already_done,
                    &mut later_queue,
                )
            }
            let values = self
                .receiver
                .drain()
                .collect::<Vec<LightPropagationRequest>>();
            for v in values {
                match v {
                    LightPropagationRequest::ChunkSky { position, world } => {
                        self.skylight_for(
                            &mut propagator,
                            position,
                            world,
                            &mut already_done,
                            &mut later_queue,
                        );
                    }
                    LightPropagationRequest::BlockLight { position, was_source } => {
                        self.blocklight_for(&mut propagator, position, was_source);
                    }
                }
            }

            if num % 100 == 0 {
                num = 0;
                propagator.map.remove_unloaded();
            }
            num += 1;
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
        self.0.insert(p, chunk);
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
pub struct LockedChunkMap(pub (), pub RegChunkMap, pub FxHashMap<ChunkCoords, usize>);

impl LockedChunkMap {
    /// Creates a new, empty world.
    pub fn new() -> Self {
        Self::default()
    }
    pub fn remove_unloaded(&mut self) {
        self.1 .0.retain(|k, v| {
            let loaded = v.is_loaded();
            if !loaded {
                self.2.remove(k);
            }
            loaded
        });
    }

    /// Retrieves a handle to the chunk at the given
    /// position, or `None` if it is not loaded.
    fn chunk_at(
        &mut self,
        pos: ChunkCoords,
        manager: &mut LightThreadManager,
    ) -> Option<RwLockWriteGuard<Chunk>> {
        self.1.chunk_at_mut(pos)
    }
    pub fn add_to_regmap(&mut self, pos: ChunkCoords, manager: &mut LightThreadManager) {
        if !self.1.has_chunk(pos) {
            let count = self.2.get(&pos).cloned().unwrap_or(0);
            if count < 3 {
                // more than 3 failed responses, accept that it does not exist
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

    pub fn block_at(
        &mut self,
        pos: BlockPosition,
        manager: &mut LightThreadManager,
    ) -> Option<BlockState> {
        if self.1.has_chunk(pos.to_chunk_coords()) {
            check_coords(pos)?;

            let (x, y, z) = chunk_relative_pos(pos.into());
            self.chunk_at(pos.to_chunk_coords(), manager)
                .map(|chunk| chunk.block_at(x, y, z))
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
        if self.1.has_chunk(pos.to_chunk_coords()) {
            if check_coords(pos).is_none() {
                return false;
            }
            let (x, y, z) = chunk_relative_pos(pos.into());
            if let Some(mut chunk) = self.chunk_at(pos.to_chunk_coords(), manager) {
                chunk.set_block_at_nlh(x, y, z, block);
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
        self.1.insert_chunk(handle);
    }

    /// Removes the chunk at the given position, returning `true` if it existed.
    pub fn remove_chunk(&mut self, pos: ChunkCoords) -> bool {
        self.1.remove_chunk(pos)
    }
}

pub struct LightThreadSync {
    pub receiver: Receiver<GameCommand>,
    pub sender: Sender<LightPropagationRequest>,
}
pub struct LightPropagator {
    queue: VecDeque<(BlockPosition, u8, bool)>,
    decrease_queue: VecDeque<(BlockPosition, u8)>,
    map: LockedChunkMap,
}
impl LightPropagator {
    pub fn clear_map(&mut self) {
        self.map = LockedChunkMap::new();
    }
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            map: LockedChunkMap::new(),
            decrease_queue: VecDeque::new()
        }
    }
    pub fn increase_light(
        &mut self,
        game: &mut LightThreadManager,
        position: BlockPosition,
        value: u8,
        sky_light: bool,
    ) {
        if value > 15 {
            return;
        }
        if let Some(mut state) = self.map.block_at(position, game) {
            let light = match sky_light {
                true => state.b_skylight,
                false => state.b_light,
            };
            if light < value || sky_light {
                self.queue.push_back((position, value, sky_light));
                match sky_light {
                    true => state.b_skylight = value,
                    false => state.b_light = value,
                }
                self.map.set_block_at(position, state, game);
                self.propagate(game);
            }
        }
    }

    pub fn decrease_light(
        &mut self,
        game: &mut LightThreadManager,
        position: BlockPosition,
        value: u8,
    ) {
        if value > 15 {
            return;
        }
        if let Some(mut state) = self.map.block_at(position, game) {
            self.decrease_queue.push_back((position, value));
            self.map.set_block_at(position, state, game);
            self.depropagate(game);
        }
    }


    fn depropagate(&mut self, game: &mut LightThreadManager) {
        let mut registry = ItemRegistry::global();

        let mut visited = FxHashSet::default();

        let mut to_update = vec![];
        while !self.decrease_queue.is_empty() {
            //log::info!("In queue");
            let (pos, light_value) = self.decrease_queue.pop_front().unwrap();
            if visited.contains(&pos) {
                continue;
            }
            if light_value == 0 {
                continue;
            }
            visited.insert(pos);

            self.map.lock(pos.to_chunk_coords(), game);
            if let Some(self_state) = self.map.block_at(pos, game) {

                for face in Face::all_faces() {
                    let neighbor = face.offset(pos);
                    self.map.lock(neighbor.to_chunk_coords(), game);
                    if let Some(neighbor_state) = self.map.block_at(neighbor, game) {
                        let current_level = neighbor_state.b_light;
                        //log::info!("Neighbor on {:?} is {}", face, current_level);
                        
                        if current_level >= self_state.b_light {
                            // probably being lit by somebody else?
                            to_update.push((neighbor, 0));
                        }
                        if let Some(mut bstate) = self.map.block_at(neighbor, game) {
                            bstate.b_light = 0;
                            self.map.set_block_at(neighbor, bstate, game);
                        }
                        self.decrease_queue.push_back((neighbor, light_value - 1));
                    }
                }

            }

        }
        for (position, was_source) in to_update {
            let _ = game.self_sender.send(LightPropagationRequest::BlockLight { position, was_source });
        }
    }


    fn propagate(&mut self, game: &mut LightThreadManager) {
        let mut registry = ItemRegistry::global();
        while !self.queue.is_empty() {
            //log::info!("In queue");
            let (pos, light_value, sky_light) = self.queue.pop_front().unwrap();
            for face in Face::all_faces() {
                let neighbor = face.offset(pos);
                self.map.lock(neighbor.to_chunk_coords(), game);
                if let Some(neighbor_state) = self.map.block_at(neighbor, game) {
                    let current_level;
                    current_level = match sky_light {
                        true => neighbor_state.b_skylight,
                        false => neighbor_state.b_light,
                    };
                    //log::info!("Neighbor on {:?} is {}", face, current_level);
                    if current_level >= (light_value - 1) {
                        continue;
                    }
                    let mut target_level = light_value.saturating_sub(
                        1.max(Self::opacity(neighbor_state.b_type, registry.clone())),
                    );
                    if target_level > 15 {
                        target_level = 0;
                    }
                    if target_level > current_level {
                        if let Some(mut bstate) = self.map.block_at(neighbor, game) {
                            match sky_light {
                                true => bstate.b_skylight = target_level,
                                false => bstate.b_light = target_level,
                            }
                            self.map.set_block_at(neighbor, bstate, game);
                        }
                        self.queue.push_back((neighbor, target_level, sky_light));
                    }
                }
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
    if game.is_solid_block(position) {
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
