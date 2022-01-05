//! Chunk loading and unloading based on player `View`s.

use std::{
    collections::{HashMap, VecDeque},
    mem,
    time::{Duration, Instant},
};

use ahash::AHashMap;
use hecs::Entity;
use nbt::CompoundTag;

use crate::{
    block_entity::{BlockEntity, BlockEntityLoader, BlockEntityNBTLoaders, SignData},
    ecs::{
        entities::player::{CurrentWorldInfo, PreviousWorldInfo},
        systems::{SysResult, SystemExecutor},
    },
    entities::EntityInit,
    events::{DeferredSpawnEvent, EntityRemoveEvent, ViewUpdateEvent},
    game::{BlockPosition, ChunkCoords, Game, Position},
    server::Server,
    world::{chunk_subscriptions::vec_remove_item, worker::LoadRequest},
};

use super::light::LightPropagationManager;

pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    let mut state = GlobalChunkLoadState::default();
    for (world, _) in game.worlds.iter() {
        state.states.insert(*world, ChunkLoadState::default());
    }
    game.insert_object(state);
    systems
        .group::<GlobalChunkLoadState>()
        .add_system(remove_dead_entities)
        .add_system(update_tickets_for_players)
        .add_system(unload_chunks)
        .add_system(load_chunks);
}

/// Amount of time to wait after a chunk has
/// no tickets until it is unloaded.
const UNLOAD_DELAY: Duration = Duration::from_secs(10);

#[derive(Default)]
struct GlobalChunkLoadState {
    pub states: AHashMap<i32, ChunkLoadState>,
}
impl GlobalChunkLoadState {
    pub fn get_mut(&mut self, idx: i32) -> anyhow::Result<&mut ChunkLoadState> {
        if let Some(state) = self.states.get_mut(&idx) {
            Ok(state)
        } else {
            Err(anyhow::anyhow!("No state for world {}", idx))
        }
    }
}

#[derive(Default)]
struct ChunkLoadState {
    /// Chunks that have been queued for unloading.
    chunk_unload_queue: VecDeque<QueuedChunkUnload>,

    chunk_tickets: ChunkTickets,
}

impl ChunkLoadState {
    pub fn remove_ticket(&mut self, chunk: ChunkCoords, ticket: Ticket) -> Option<()> {
        self.chunk_tickets.remove_ticket(chunk, ticket)?;

        // If this was the last ticket, then queue the chunk to be
        // unloaded.
        if self.chunk_tickets.num_tickets(chunk) == 0 {
            //log::info!("Chunk {:?} IS worthy of unload", chunk);
            self.chunk_tickets.remove_chunk(chunk);
            self.chunk_unload_queue
                .push_back(QueuedChunkUnload::new(chunk));
        } else {
            //log::info!("Chunk {:?} not worthy of unload", chunk);
        }
        Some(())
    }
}

#[derive(Copy, Clone, Debug)]
struct QueuedChunkUnload {
    pos: ChunkCoords,
    /// Time after which the chunk should be unloaded.
    unload_at_time: Instant,
}

impl QueuedChunkUnload {
    pub fn new(pos: ChunkCoords) -> Self {
        Self {
            pos,
            unload_at_time: Instant::now() + UNLOAD_DELAY,
        }
    }
}

/// Maintains a list of "tickets" for each loaded chunk.
/// A chunk is queued for unloading when it has no more tickets.
#[derive(Default)]
struct ChunkTickets {
    tickets: AHashMap<ChunkCoords, Vec<Ticket>>,
    by_entity: AHashMap<Ticket, Vec<ChunkCoords>>,
}

impl ChunkTickets {
    pub fn insert_ticket(&mut self, chunk: ChunkCoords, ticket: Ticket) {
        self.tickets.entry(chunk).or_default().push(ticket);
        self.by_entity.entry(ticket).or_default().push(chunk);
    }

    pub fn remove_ticket(&mut self, chunk: ChunkCoords, ticket: Ticket) -> Option<()> {
        if let Some(vec) = self.tickets.get_mut(&chunk) {
            vec_remove_item(vec, &ticket);
        }
        vec_remove_item(self.by_entity.get_mut(&ticket)?, &chunk);
        Some(())
    }

    pub fn num_tickets(&self, chunk: ChunkCoords) -> usize {
        match self.tickets.get(&chunk) {
            Some(vec) => vec.len(),
            None => 0,
        }
    }

    pub fn take_entity_tickets(&mut self, ticket: Ticket) -> Vec<ChunkCoords> {
        self.by_entity
            .get_mut(&ticket)
            .map(mem::take)
            .unwrap_or_default()
    }

    pub fn remove_chunk(&mut self, pos: ChunkCoords) {
        self.tickets.remove(&pos);
    }
}

/// ID of a chunk ticket that keeps a chunk loaded.
///
/// Currently just represents an entity, the player
/// that is keeping this chunk loaded.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct Ticket(Entity);

/// System to populate chunk tickets based on players' views.
fn update_tickets_for_players(game: &mut Game, gl_state: &mut GlobalChunkLoadState) -> SysResult {
    for (_, world) in game.worlds.iter_mut() {
        for (player, (event, current_world, prev_world)) in game
            .ecs
            .query::<(&ViewUpdateEvent, &CurrentWorldInfo, &mut PreviousWorldInfo)>()
            .iter()
        {
            let state = gl_state.get_mut(current_world.world_id)?;
            let player_ticket = Ticket(player);

            // Remove old tickets
            let mut flag = false;
            for &old_chunk in &event.old_chunks {
                if state.remove_ticket(old_chunk, player_ticket).is_none() {
                    flag = true;
                    break;
                }
            }
            if prev_world.1.world_id != current_world.world_id {
                //log::info!("Differing!");
                let state = gl_state.get_mut(prev_world.1.world_id)?;
                prev_world.1.world_id = current_world.world_id;
                for &old_chunk in &event.old_chunks {
                    state.remove_ticket(old_chunk, player_ticket);
                }
            }
            let state = gl_state.get_mut(current_world.world_id)?;
            if current_world.world_id == world.id {
                // Create new tickets
                for &new_chunk in &event.new_chunks {
                    state.chunk_tickets.insert_ticket(new_chunk, player_ticket);

                    // Load if needed
                    if !world.is_chunk_loaded(&new_chunk) && !world.is_chunk_loading(&new_chunk) {
                        world.queue_chunk_load(LoadRequest { pos: new_chunk });
                    }
                }
            }
        }
    }
    Ok(())
}

/// System to unload chunks from the `ChunkUnloadQueue`.
fn unload_chunks(game: &mut Game, state: &mut GlobalChunkLoadState) -> SysResult {
    for (_, world) in game.worlds.iter_mut() {
        let mut state = state.get_mut(world.id)?;
        while let Some(&unload) = state.chunk_unload_queue.get(0) {
            if unload.unload_at_time > Instant::now() {
                // None of the remaining chunks in the queue are
                // ready for unloading, because the queue is ordered
                // by time.
                break;
            }

            state.chunk_unload_queue.pop_front();

            // If the chunk has acquired new tickets, then abort unloading it.
            if state.chunk_tickets.num_tickets(unload.pos) > 0 {
                continue;
            }

            world.unload_chunk(&mut game.ecs, &unload.pos)?;
        }
        world.cache.purge_unused();
    }
    Ok(())
}

fn remove_dead_entities(game: &mut Game, state: &mut GlobalChunkLoadState) -> SysResult {
    for (entity, (_event, world, prev_world)) in game
        .ecs
        .query::<(&EntityRemoveEvent, &CurrentWorldInfo, &PreviousWorldInfo)>()
        .iter()
    {
        let mut state = state.get_mut(world.world_id)?;
        let entity_ticket = Ticket(entity);
        for chunk in state.chunk_tickets.take_entity_tickets(entity_ticket) {
            state.remove_ticket(chunk, entity_ticket);
        }
    }
    Ok(())
}

/// System to call `World::load_chunks` each tick
fn load_chunks(game: &mut Game, chunk_load_state: &mut GlobalChunkLoadState) -> SysResult {
    let be_nbt = game.objects.get::<BlockEntityNBTLoaders>()?.clone();
    let mut te_data: AHashMap<i32, Vec<CompoundTag>> = AHashMap::new();
    let mut light = game.objects.get_mut::<LightPropagationManager>()?;
    for (id, world) in game.worlds.iter_mut() {
        if let Some(vec) = te_data.get_mut(id) {
            vec.append(&mut world.load_chunks(&mut game.ecs, &mut light)?);
        } else {
            let mut vec = Vec::new();
            vec.append(&mut world.load_chunks(&mut game.ecs, &mut light)?);
            te_data.insert(*id, vec);
        }
    }
    drop(light);
    for (world_id, tags) in te_data {
        for tag in tags {
            let id = tag
                .get_str("id")
                .or_else(|_| Err(anyhow::anyhow!("No tag")))?
                .to_string();
            let x = tag
                .get_i32("x")
                .or_else(|_| Err(anyhow::anyhow!("No tag")))?;
            let y = tag
                .get_i32("y")
                .or_else(|_| Err(anyhow::anyhow!("No tag")))?;
            let z = tag
                .get_i32("z")
                .or_else(|_| Err(anyhow::anyhow!("No tag")))?;
            let pos = BlockPosition::new(x, y, z, world_id);
            game.remove_block_entity_at(pos, 0)?;
            let pospos = Position::from_pos(x as f64, y as f64, z as f64, world_id);
            let mut builder = game.create_entity_builder(pospos, EntityInit::BlockEntity);
            // TODO do multiworld
            builder.add(BlockEntity(pos, 0));
            if be_nbt.run(id.clone(), &tag, pos, &mut builder) {
                game.ecs.insert_event(DeferredSpawnEvent(builder));
            /*             let entity_ref = game.ecs.entity(e)?;
            let server = game.objects.get::<Server>()?;
            if let Ok(loader) = entity_ref.get::<BlockEntityLoader>() {
                log::info!("Syncing {:?}", *entity_ref.get::<SignData>()?);
                server.sync_block_entity(pospos, (*loader).clone(), &entity_ref);
            } */
            } else {
                log::info!("No parser for type {}", id);
            }
        }
    }
    Ok(())
}
