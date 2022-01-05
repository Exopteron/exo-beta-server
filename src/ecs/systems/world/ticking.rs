use std::{
    collections::{HashMap, VecDeque},
    mem,
    time::{Duration, Instant},
};

use ahash::AHashMap;
use hecs::Entity;
use rand::Rng;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    block_entity::{BlockEntity, BlockEntityLoader, BlockEntityNBTLoaders, SignData},
    ecs::{
        entities::player::Player,
        systems::{SysResult, SystemExecutor},
    },
    entities::EntityInit,
    events::{DeferredSpawnEvent, EntityRemoveEvent, ViewUpdateEvent},
    game::{BlockPosition, ChunkCoords, Game, Position},
    item::item::{block::AtomicRegistryBlock, BlockIdentifier, ItemRegistry},
    server::Server,
    world::{
        chunk_map::chunk_relative_pos, chunk_subscriptions::vec_remove_item, worker::LoadRequest,
    },
};
pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    systems.add_system(tick_blocks);
}
// TODO: make this better. it's really slow at the moment
pub fn tick_blocks(game: &mut Game) -> SysResult {
    let mut rng = rand::thread_rng();
    let mut to_tick = AHashMap::new();
    for (world_id, world) in game.worlds.iter() {
        let mut loaded = world.loaded_chunks();
        loaded.retain(|coords| {
            for (_, (_, position)) in game.ecs.query::<(&Player, &Position)>().iter() {
                if position.to_chunk_coords().distance_squared_to(*coords) < 8 {
                    return true;
                }
            }
            false
        });
        to_tick.insert(*world_id, loaded);
    }
    let registry = ItemRegistry::global();
    for (world_id, loaded) in to_tick {
        loaded.iter().for_each(|chunkpos| {
            //log::info!("Ticking chunk {}", chunk);
            for section in 0..8 {
                for _ in 0..3 {
                    let x = rng.gen_range(0..16);
                    let y = rng.gen_range(0..16) + (section * 16);
                    let z = rng.gen_range(0..16);
                    let block_pos = BlockPosition::new(x, y, z, world_id);
                    //log::info!("Ticking {:?} in chunk {}", block_pos, chunk);
                    let world = game.worlds.get(&world_id).unwrap();
                    let chunk = match world.chunk_map.chunk_at(*chunkpos) {
                        Some(v) => v,
                        None => {
                            continue;
                        }
                    };
                    let (x, y, z) = chunk_relative_pos(block_pos);
                    let block_state = match chunk.block_at(x, y, z) {
                        Some(v) => v,
                        None => continue,
                    };
                    drop(chunk);
                    let block_pos = BlockPosition::new(x as i32 + (chunkpos.x * 16) , y as i32, z as i32 + (chunkpos.z * 16), world_id);
                    let start = Instant::now();
                    if let Some(block_type) = registry.get_block(block_state.b_type) {
                        let ms = start.elapsed().as_millis();
                        if ms > 0 {
                            //log::info!("Ticking took {}ms", ms);
                        }
                        block_type.tick(world_id, game, block_state, block_pos);
                    }
                }
            }
        });
    }
    Ok(())
}
