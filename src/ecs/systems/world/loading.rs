//! Chunk loading and unloading based on player `View`s.

use std::{
    collections::{VecDeque, HashMap},
    mem,
    time::{Duration, Instant},
};


use hecs::Entity;

use crate::{
    events::{EntityRemoveEvent, ViewUpdateEvent}, ecs::systems::{SystemExecutor, SysResult}, game::{Game, ChunkCoords}, world::chunk_subscriptions::vec_remove_item,
};

pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    game.insert_object(ChunkLoadState::default());
    systems
        .group::<ChunkLoadState>()
        .add_system(remove_dead_entities)
        .add_system(update_tickets_for_players)
        .add_system(unload_chunks)
        .add_system(load_chunks);
}

/// Amount of time to wait after a chunk has
/// no tickets until it is unloaded.
const UNLOAD_DELAY: Duration = Duration::from_secs(10);

#[derive(Default)]
struct ChunkLoadState {
    /// Chunks that have been queued for unloading.
    chunk_unload_queue: VecDeque<QueuedChunkUnload>,

    chunk_tickets: ChunkTickets,
}

impl ChunkLoadState {
    pub fn remove_ticket(&mut self, chunk: ChunkCoords, ticket: Ticket) {
        self.chunk_tickets.remove_ticket(chunk, ticket);

        // If this was the last ticket, then queue the chunk to be
        // unloaded.
        if self.chunk_tickets.num_tickets(chunk) == 0 {
            self.chunk_tickets.remove_chunk(chunk);
            self.chunk_unload_queue
                .push_back(QueuedChunkUnload::new(chunk));
        }
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
    tickets: HashMap<ChunkCoords, Vec<Ticket>>,
    by_entity: HashMap<Ticket, Vec<ChunkCoords>>,
}

impl ChunkTickets {
    pub fn insert_ticket(&mut self, chunk: ChunkCoords, ticket: Ticket) {
        self.tickets.entry(chunk).or_default().push(ticket);
        self.by_entity.entry(ticket).or_default().push(chunk);
    }

    pub fn remove_ticket(&mut self, chunk: ChunkCoords, ticket: Ticket) {
        if let Some(vec) = self.tickets.get_mut(&chunk) {
            vec_remove_item(vec, &ticket);
        }
        vec_remove_item(self.by_entity.get_mut(&ticket).unwrap(), &chunk);
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
fn update_tickets_for_players(game: &mut Game, state: &mut ChunkLoadState) -> SysResult {
    for (_, world) in game.worlds.iter_mut() {
        for (player, event) in game.ecs.query::<&ViewUpdateEvent>().iter() {
            let player_ticket = Ticket(player);
    
            // Remove old tickets
            for &old_chunk in &event.old_chunks {
                state.remove_ticket(old_chunk, player_ticket);
            }
    
            // Create new tickets
            for &new_chunk in &event.new_chunks {
                state.chunk_tickets.insert_ticket(new_chunk, player_ticket);
    
                // Load if needed
                if !world.is_chunk_loaded(&new_chunk) && !world.is_chunk_loading(&new_chunk) {
                    world.queue_chunk_load(&new_chunk);
                }
            }
        }
    }
    Ok(())
}

/// System to unload chunks from the `ChunkUnloadQueue`.
fn unload_chunks(game: &mut Game, state: &mut ChunkLoadState) -> SysResult {
    for (_, world) in game.worlds.iter_mut() {
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
    
            world.unload_chunk(&unload.pos)?;
        }
    }
    Ok(())
}

fn remove_dead_entities(game: &mut Game, state: &mut ChunkLoadState) -> SysResult {
    for (entity, _event) in game.ecs.query::<&EntityRemoveEvent>().iter() {
        let entity_ticket = Ticket(entity);
        for chunk in state.chunk_tickets.take_entity_tickets(entity_ticket) {
            state.remove_ticket(chunk, entity_ticket);
        }
    }
    Ok(())
}

/// System to call `World::load_chunks` each tick
fn load_chunks(game: &mut Game, chunk_load_state: &mut ChunkLoadState) -> SysResult {
    for (_, world) in game.worlds.iter_mut() {
        world.load_chunks(&mut game.ecs)?;
    }
    Ok(())
}