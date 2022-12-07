//! Implements block change broadcasting.
//!
//! # Bulk updates
//! The protocol provides three methods to change blocks
//! on the client:
//! * The `BlockChange` packet to update a single block.
//! * The `MultiBlockChange` packet to update multiple blocks
//! within a single chunk section.
//! * The `ChunkData` packet to overwrite entire chunk sections
//! at once.
//!
//! Feather is optimized for bulk block updates to cater to plugins
//! like WorldEdit. This module chooses the optimal packet from
//! the above three options to achieve ideal performance.

use ahash::AHashMap;
mod place;
pub mod update;
use crate::{ecs::{systems::{SystemExecutor, SysResult}, entities::player::Username}, game::{Game, Position}, server::Server, world::chunks::{SECTION_VOLUME, BlockState}, events::block_change::BlockChangeEvent, protocol::packets::Face, item::item::ItemRegistry, block_entity::{BlockEntity, BlockEntityLoader}, entities::EntityInit};

use self::update::BlockUpdateManager;

pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    systems
        .group::<Server>()
        .add_system(broadcast_block_changes);
    place::register(game, systems);
    update::register(game, systems);
}

fn broadcast_block_changes(game: &mut Game, server: &mut Server) -> SysResult {
    let mut to_update = Vec::new();
    let mut to_add_new = Vec::new();
    for (_, event) in game.ecs.query::<&BlockChangeEvent>().iter() {
        if event.update_self || event.update_neighbors {
            for block in event.iter_changed_blocks() {
                to_update.push((block, event.world(), event.update_neighbors, event.update_self));
            }
        }
        for update in event.iter_changed_blocks() {
            to_add_new.push((update, event.world()));
        }
        broadcast_block_change(event, game, server);
    }
    for update in to_add_new.iter() {
        if let Some(blockstate) = game.block(update.0, update.1) {
            if let Ok(block) = blockstate.registry_type() {
                block.added(update.1, game, server, update.0, blockstate);
            }
        }
    }
    for update in to_update.iter() {
        game.remove_block_entity_at(update.0, update.1)?;
        if let Some(blockstate) = game.block(update.0, update.1) {
            if let Ok(block) = blockstate.registry_type() {
                let mut builder = game.create_entity_builder(update.0.into(), EntityInit::BlockEntity);
                builder.add(BlockEntity(update.0, update.1));
                if block.block_entity(&mut builder, blockstate, update.0.clone()) {
                    game.spawn_entity(builder);
                }
            }
        }
    }
    let mut manager = game.objects.get_mut::<BlockUpdateManager>()?;
    for update in to_update {
        manager.add(update);
    }
    Ok(())
}

/// Threshold at which to switch from block change to chunk
// overwrite packets.
const CHUNK_OVERWRITE_THRESHOLD: usize = SECTION_VOLUME / 2;

fn broadcast_block_change(event: &BlockChangeEvent, game: &Game, server: &mut Server) {
    if event.count() >= CHUNK_OVERWRITE_THRESHOLD {
        broadcast_block_change_chunk_overwrite(event, game, server);
    } else {
        broadcast_block_change_simple(event, game, server);
    }
}

fn broadcast_block_change_chunk_overwrite(
    event: &BlockChangeEvent,
    game: &Game,
    server: &mut Server,
) {
    todo!()
/*     let mut sections: AHashMap<ChunkPosition, Vec<usize>> = AHashMap::new();
    for (chunk, section, _) in event.iter_affected_chunk_sections() {
        sections.entry(chunk).or_default().push(section + 1); // + 1 to account for the void air chunk
    }

    for (chunk_pos, sections) in sections {
        let chunk = game.world.chunk_map().chunk_handle_at(chunk_pos);
        if let Some(chunk) = chunk {
            let position = position!(
                (chunk_pos.x * CHUNK_WIDTH as i32) as f64,
                0.0,
                (chunk_pos.z * CHUNK_WIDTH as i32) as f64,
            );
            server.broadcast_nearby_with(position, |client| {
                client.overwrite_chunk_sections(&chunk, sections.clone());
            })
        }
    } */
}

fn broadcast_block_change_simple(event: &BlockChangeEvent, game: &Game, server: &mut Server) {
    for pos in event.iter_changed_blocks() {
        //log::info!("UPDATEPROP pos: {:?}", pos);
        let new_block = game.block(pos, event.world());
        if let Some(new_block) = new_block {
            //let new_block = BlockState::from_id(7);
            //log::info!("UPDATEPROP Setting {:?} to {:?}", pos, new_block);
            server.broadcast_nearby_with(pos.into(), |client| {
                client.send_block_change(pos, new_block);
            });
        }
    }
}
