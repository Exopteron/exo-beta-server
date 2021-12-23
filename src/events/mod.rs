use hecs::EntityBuilder;

use crate::{world::{view::View, chunk_lock::ChunkHandle}, game::ChunkCoords};

// from feather license in FEATHER_LICENSE.md
pub mod block_change;
pub mod block_interact;
/// Triggered when an entity is removed from the world.
///
/// The entity will remain alive for one tick after it is
/// destroyed to allow systems to observe this event.
#[derive(Debug)]
pub struct EntityRemoveEvent;

#[derive(Debug)]
pub struct EntityDeathEvent;

/// Event triggered when an entity crosses into a new chunk.
///
/// Unlike [`ViewUpdateEvent`], this event triggers for all entities,
/// not just players.
pub struct ChunkCrossEvent {
    pub old_chunk: ChunkCoords,
    pub new_chunk: ChunkCoords,
}


/// Triggered when an entity is added into the world.
#[derive(Debug)]
pub struct EntityCreateEvent;

/// Event triggered when a player changes their `View`,
/// meaning they crossed into a new chunk.
#[derive(Debug)]
pub struct ViewUpdateEvent {
    pub old_view: View,
    pub new_view: View,

    /// Chunks that are in `new_view` but not `old_view`
    pub new_chunks: Vec<ChunkCoords>,
    /// Chunks that are in `old_view` but not in `new_view`
    pub old_chunks: Vec<ChunkCoords>,
}

impl ViewUpdateEvent {
    pub fn new(old_view: View, new_view: View) -> Self {
        let mut this = Self {
            old_view,
            new_view,
            new_chunks: new_view.difference(old_view).collect(),
            old_chunks: old_view.difference(new_view).collect(),
        };
        this.new_chunks
            .sort_unstable_by_key(|chunk| chunk.distance_squared_to(new_view.center()));
        this.old_chunks
            .sort_unstable_by_key(|chunk| chunk.distance_squared_to(old_view.center()));
        this
    }
}

/// Triggered when a player joins the `Game`.
#[derive(Debug)]
pub struct PlayerJoinEvent;

#[derive(Debug)]
pub struct PlayerSpawnEvent;
/// Triggered when a chunk is loaded.
#[derive(Debug)]
pub struct ChunkLoadEvent {
    pub position: ChunkCoords,
    pub chunk: ChunkHandle,
}

/// Triggered when an error occurs while loading a chunk.
#[derive(Debug)]
pub struct ChunkLoadFailEvent {
    pub position: ChunkCoords,
}

#[derive(Debug, Clone)]
pub struct SneakEvent {
    pub is_sneaking: bool,
}

impl SneakEvent {
    pub fn new(changed_to: bool) -> Self {
        Self {
            is_sneaking: changed_to,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SprintEvent {
    pub is_sprinting: bool,
}

impl SprintEvent {
    pub fn new(changed_to: bool) -> Self {
        Self {
            is_sprinting: changed_to,
        }
    }
}


pub struct DeferredSpawnEvent(pub EntityBuilder);