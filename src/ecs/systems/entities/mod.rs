use std::mem;

use crate::{game::Game, server::Server, events::DeferredSpawnEvent, status_effects::StatusEffectsManager};

use super::{SystemExecutor, SysResult};

pub mod player;
pub mod item;
pub mod falling_block;
pub mod living;
pub fn default_systems(_game: &mut Game, systems: &mut SystemExecutor<Game>) {
    falling_block::init_systems(systems);
    item::init_systems(systems);
    living::init_systems(systems);
    systems.add_system(deferred_spawn);
    systems.group::<Server>().add_system(tick_clients).add_system(StatusEffectsManager::system);
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