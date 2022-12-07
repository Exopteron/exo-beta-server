use std::{vec::Drain, mem};

use crate::{game::{BlockPosition, Game}, ecs::systems::{SystemExecutor, SysResult}, protocol::packets::Face, item::item::ItemRegistry};

pub struct BlockUpdateManager {
    updates: Vec<(BlockPosition, i32, bool, bool)>
}
impl BlockUpdateManager {
    pub fn new() -> Self {
        Self { updates: Vec::new() }
    }
    pub fn add(&mut self, update: (BlockPosition, i32, bool, bool)) {
        self.updates.push(update);
    }
    pub fn take_queue(&mut self) -> Vec<(BlockPosition, i32, bool, bool)> {
        mem::take(&mut self.updates)
    }
}
pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    systems.add_system(clear_queue);
}

pub fn clear_queue(game: &mut Game) -> SysResult {
    let mut manager = game.objects.get_mut::<BlockUpdateManager>()?;
    let queue = manager.take_queue();
    drop(manager);
    for entry in queue {
        let (block, world, update_neighbors, update_self) = entry;
        let origin_state = game.block(block, world).expect("? Where did the block go?");
        let block_type = ItemRegistry::global().get_block(origin_state.b_type);
        if update_self {
            if let Some(block_type) = block_type {
                //log::info!("Updating neighbor at {:?} from {:?}", block, origin);
                if let Err(_) = block_type.neighbor_update(world, game, block, origin_state, Face::Invalid, origin_state) {
                    // TODO handle
                }
                //to_update.push((block, world));
            }
        }
    }
    Ok(())
}