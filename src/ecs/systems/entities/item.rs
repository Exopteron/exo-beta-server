use crate::{ecs::{systems::{SystemExecutor, SysResult}, entities::{item::{ItemEntity, Life}, player::Player}}, game::{Game, Position}, physics::Physics};


pub fn init_systems(s: &mut SystemExecutor<Game>) {
    s.add_system(Physics::system).add_system(epic_system).add_system(pickup_items).add_system(increment_life);
}

fn increment_life(game: &mut Game) -> SysResult {
    for (_, life) in game.ecs.query::<&mut Life>().iter() {
        life.0 += 1;
    }
    Ok(())
}

fn epic_system(game: &mut Game) -> SysResult {
    for (_, (_, physics)) in game.ecs.query::<(&ItemEntity, &mut Physics)>().iter() {
        physics.add_velocity(0., -1., 0.);
    }
    Ok(())
}

fn pickup_items(game: &mut Game) -> SysResult {
    let mut item_entities = Vec::new();
    for (entity, (_, pos, life)) in game.ecs.query::<(&ItemEntity, &Position, &Life)>().iter() {
        if life.0 > 15 {
            item_entities.push((entity, *pos));
        }
    }
    let mut to_despawn = Vec::new();
    for (_, (_, pos)) in game.ecs.query::<(&Player, &Position)>().iter() {
        for (entity, item_pos) in item_entities.iter() {
            log::info!("Distance: {:?}", pos.distance(item_pos));
            if pos.world == item_pos.world && pos.distance(item_pos) < 0.1 {
                to_despawn.push(entity);
            }
        }    
    }
    for entity in to_despawn {
        game.remove_entity(*entity)?;
    }
    Ok(())
}