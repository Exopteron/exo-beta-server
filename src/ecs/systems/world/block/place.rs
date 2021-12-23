use crate::{game::{Game, BlockPosition}, ecs::systems::{SystemExecutor, SysResult}, server::Server, events::block_interact::BlockPlacementEvent, protocol::packets::Face, world::chunks::BlockState, item::stack::ItemStackType};

use super::update::BlockUpdateManager;

pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    systems.add_system(do_block_placement);
}

fn do_block_placement(game: &mut Game) -> SysResult {
    let mut blocks = Vec::new();
    for (_, event) in game.ecs.query::<&BlockPlacementEvent>().iter() {
        let id = match event.held_item.item() {
            ItemStackType::Item(_) => continue,
            ItemStackType::Block(b) => {
                b.id()
            }
        };
        let block = BlockState { b_type: id.0, b_metadata: event.held_item.damage_taken() as u8, b_light: 0, b_skylight: 15 };
        blocks.push((event.location, block, event.world));
    }
    for (pos, block, world) in blocks.iter() {
        game.set_block(*pos, *block, *world);
    }
    let mut update_manager = game.objects.get_mut::<BlockUpdateManager>()?;
    for (pos, _, world) in blocks {
        update_manager.add((pos, world));
    }
    Ok(())
}