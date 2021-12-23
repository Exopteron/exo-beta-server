use std::{vec::Drain, mem};

use crate::{game::{BlockPosition, Game}, ecs::systems::{SystemExecutor, SysResult}, protocol::packets::Face, item::item::ItemRegistry};

pub struct BlockUpdateManager {
    updates: Vec<(BlockPosition, i32)>
}
impl BlockUpdateManager {
    pub fn new() -> Self {
        Self { updates: Vec::new() }
    }
    pub fn add(&mut self, update: (BlockPosition, i32)) {
        self.updates.push(update);
    }
    pub fn take_queue(&mut self) -> Vec<(BlockPosition, i32)> {
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
        let (block, world) = entry;
        let origin_state = game.block(block, world).expect("? Where did the block go?");
        let block_type = ItemRegistry::global().get_block((origin_state.b_type, 0));
        if let Some(block_type) = block_type {
            //log::info!("Updating neighbor at {:?} from {:?}", block, origin);
            if let Err(_) = block_type.neighbor_update(world, game, block, origin_state, Face::Invalid, origin_state) {
                // TODO handle
            }
            //to_update.push((block, world));
        }
        for face in Face::all_faces() {
            let origin = block.clone();
            //log::info!("Face: {:?} w original: {:?}", face, block);
            let block = face.offset(block);
            //log::info!("Offset: {:?}", block);
            let block_state = game.block(block, world).expect("? No block again?");
            // TODO: Should we have different meta values update different states? Wool etc
            if block_state.b_type == 0 {
                continue;
            }
            let block_type = ItemRegistry::global().get_block((block_state.b_type, 0));
            if let Some(block_type) = block_type {
                //log::info!("Updating neighbor at {:?} from {:?}", block, origin);
                if let Err(_) = block_type.neighbor_update(world, game, block, block_state, face, origin_state) {
                    // TODO handle
                }
                //to_update.push((block, world));
            }
        }
    }
    Ok(())
}