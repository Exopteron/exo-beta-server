use std::mem;

use hecs::Entity;

use crate::{game::{Game, BlockPosition}, server::Server, events::DeferredSpawnEvent, status_effects::StatusEffectsManager, world::chunks::BlockState, block_entity::BlockEntity};

use super::{SystemExecutor, SysResult};

pub mod player;
pub mod item;
pub mod falling_block;
pub mod living;


pub struct BlockEntityTicker(pub fn(&mut Game, &mut Server, Entity, BlockPosition, BlockState) -> SysResult);
pub fn default_systems(game: &mut Game, systems: &mut SystemExecutor<Game>) -> anyhow::Result<()> {
    falling_block::init_systems(systems);
    item::init_systems(game, systems)?;
    living::init_systems(game, systems)?;
    systems.add_system(deferred_spawn);
    systems.group::<Server>().add_system(tick_clients).add_system(StatusEffectsManager::system).add_system(tick_block_entities);
    Ok(())
}
fn tick_block_entities(game: &mut Game, server: &mut Server) -> SysResult {
    let mut entities = Vec::new();
    for (e, _) in game.ecs.query::<(&BlockEntity, &BlockEntityTicker)>().iter() {
        entities.push(e);
    }
    for e in entities {
        let ticker = game.ecs.get::<BlockEntityTicker>(e)?.0;
        let pos = game.ecs.get::<BlockEntity>(e)?.0;
        let b = match game.block(pos, pos.world) {
            Some(b) => b,
            None => continue
        };
        ticker(game, server, e, pos, b)?;
    }
    Ok(())
}

/// Ticks `Client`s.
fn tick_clients(_game: &mut Game, server: &mut Server) -> SysResult {
    for client in server.clients.iter() {
        client.1.tick();
    }
    Ok(())
}

fn deferred_spawn(game: &mut Game) -> SysResult {
    let mut deferred = Vec::new();
    for (_, event) in game.ecs.query::<&mut DeferredSpawnEvent>().iter() {
        //log::info!("Got deferred spawn event");
        deferred.push(mem::take(&mut event.0));
    }
    for builder in deferred {
        game.spawn_entity(builder);
    }
    Ok(())
}