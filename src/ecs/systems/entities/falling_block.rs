use std::ops::Deref;

use crate::{ecs::{systems::{SystemExecutor, SysResult}, entities::{falling_block::{FallingBlockEntity, FallingBlockEntityData}, item::ItemEntityBuilder}}, game::{Game, Position, BlockPosition}, physics::Physics, item::{item::ItemRegistry, stack::ItemStack}, world::chunks::BlockState, events::EntityDeathEvent};

pub fn init_systems(s: &mut SystemExecutor<Game>) {
    s.add_system(epic_system);
}

fn epic_system(game: &mut Game) -> SysResult {
    let mut fallingblocks = Vec::new();
    for (entity, (_, _, _)) in game
        .ecs
        .query::<(&FallingBlockEntity, &mut Physics, &Position)>()
        .iter()
    {
        fallingblocks.push(entity);
    }
    for entity in fallingblocks {
        let entity_ref = game.ecs.entity(entity)?;
        if entity_ref.get::<EntityDeathEvent>().is_err() {
            let mut fakephysics = entity_ref.get_mut::<Physics>()?.deref().clone();
            fakephysics.add_velocity(0., -0.05, 0.);
            drop(entity_ref);
            fakephysics.move_entity(game, entity, *fakephysics.get_velocity())?;
            let entity_ref = game.ecs.entity(entity)?;
            let mut physics = entity_ref.get_mut::<Physics>()?;
            *physics = fakephysics;
            let velocity = physics.get_velocity_mut();
            velocity.x *= 0.98;
            velocity.y *= 0.98;
            velocity.z *= 0.98;
            let real_pos = *entity_ref.get::<Position>()?;
            let block_pos: BlockPosition = (real_pos).into();
            let data = *entity_ref.get::<FallingBlockEntityData>()?;
            let mut fakephysics = physics.deref().clone();
            drop(physics);
            drop(entity_ref);
            if game.block_id_at(block_pos) == data.block_id() {
                game.break_block(block_pos, block_pos.world);
            }
            if real_pos.on_ground {
                let velocity = fakephysics.get_velocity_mut();
                velocity.x *= 0.69;
                velocity.y *= -0.5;
                velocity.z *= 0.69;
                game.remove_entity(entity)?;
                if !game.can_be_placed_at(block_pos) {
                    let mut pos = real_pos;
                    pos.on_ground = false;
                    let builder = ItemEntityBuilder::build(game, real_pos, ItemStack::new(data.block_id().into(), 1, 0), 5);
                    game.spawn_entity(builder);
                } else {
                    game.set_block(block_pos, BlockState::from_id(data.block_id()), block_pos.world);
                }
            }
            let entity_ref = game.ecs.entity(entity)?;
            let mut physics = entity_ref.get_mut::<Physics>()?;
            *physics = fakephysics;
        }
    }
    Ok(())
}