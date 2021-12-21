use crate::{game::{Game, BlockPosition}, ecs::systems::{SystemExecutor, SysResult}, server::Server, events::block_interact::BlockPlacementEvent, protocol::packets::Face, world::chunks::BlockState};

pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    systems.add_system(do_block_placement);
}

fn do_block_placement(game: &mut Game) -> SysResult {
    let mut blocks = Vec::new();
    for (_, event) in game.ecs.query::<&BlockPlacementEvent>().iter() {
        let mut pos = event.location.clone();
        offset(&mut pos, event.face.clone());
        let block = BlockState { b_type: event.held_item.item().id().0 as u8, b_metadata: event.held_item.item().id().1 as u8, b_light: 0, b_skylight: 15 };
        blocks.push((pos, block, event.world));
    }
    for (pos, block, world) in blocks {
        game.set_block(pos, block, world);
    }
    Ok(())
}

fn offset(pos: &mut BlockPosition, face: Face) {
    match face {
        Face::Invalid => panic!("Invalid face"),
        Face::NegativeY => pos.y -= 1,
        Face::PositiveY => pos.y += 1,
        Face::NegativeZ => pos.z -= 1,
        Face::PositiveZ => pos.z += 1,
        Face::NegativeX => pos.x -= 1,
        Face::PositiveX => pos.x += 1
    }
}