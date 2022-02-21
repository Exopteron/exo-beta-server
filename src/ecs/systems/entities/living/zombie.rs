use std::ops::Deref;

use glam::{Vec3, DVec3};
use hecs::Entity;

use crate::{ecs::{systems::{SystemExecutor, SysResult}, entities::{living::zombie::ZombieEntity, player::Player}}, game::{Game, BlockPosition, Position}, events::EntityDeathEvent, physics::Physics, entities::PreviousPosition, server::Server, network::ids::NetworkID};

pub fn init_systems(s: &mut SystemExecutor<Game>) {
    s.add_system(zombie_physics).add_system(fling);
    s.group::<Server>().add_system(temp);
}
fn fling(game: &mut Game) -> SysResult {
    let mut entities = Vec::new();
    for (entity, (_, position, physics)) in game.ecs.query::<(&ZombieEntity, &Position, &Physics)>().iter() {
        entities.push(entity);
    }
    for e in entities {
        let mut v = game.ecs.entity(e)?.get_mut::<usize>()?;
        *v += 1;
        if *v > 50 {
            *v = 0;
            let mut pos = *game.ecs.entity(e)?.get::<Position>()?;
            let mut closest = (1000., Position::default());
            for (_, (_, position)) in game.ecs.query::<(&Player, &Position)>().iter() {
                let dst = pos.distance(position);
                if dst < closest.0 {
                    closest = (dst, *position);
                }
            }
            let a = DVec3::new(pos.x, pos.y, pos.z);
            let b = DVec3::new(closest.1.x, closest.1.y, closest.1.z);
            let vector = b - a;
            let num = (pos.x.powf(2.) + pos.y.powf(2.) + pos.z.powf(2.)).sqrt();
            let vector = vector / num;
            let mult = 65. * vector;
            pos.x += mult.x;
            pos.y += mult.y;
            pos.z += mult.z;
            *game.ecs.entity(e)?.get_mut::<Position>()? = pos;
        }
    }
    Ok(())
}
fn temp(game: &mut Game, server: &mut Server) -> SysResult {
    let mut fallingblocks = Vec::new();
    for (entity, _) in game
        .ecs
        .query::<&ZombieEntity>()
        .iter()
    {
        fallingblocks.push(entity);
    }
    for entity in fallingblocks {
        let entity_ref = game.ecs.entity(entity)?;
        if entity_ref.get::<EntityDeathEvent>().is_err() {
            let position = *entity_ref.get::<Position>()?;
            let id = *entity_ref.get::<NetworkID>()?;
            let prev = *entity_ref.get::<PreviousPosition>()?;
            server.broadcast_nearby_with(position,  |client| {
                client.update_entity_position(
                    id,
                    position,
                    prev,
                );
            });
        }
    }
    Ok(())
}

fn zombie_physics(game: &mut Game) -> SysResult {
    let mut fallingblocks = Vec::new();
    for (entity, (_, _, _)) in game
        .ecs
        .query::<(&ZombieEntity, &mut Physics, &Position)>()
        .iter()
    {
        fallingblocks.push(entity);
    }
    for entity in fallingblocks {
        let entity_ref = game.ecs.entity(entity)?;
        if entity_ref.get::<EntityDeathEvent>().is_err() {
            let mut fakephysics = entity_ref.get_mut::<Physics>()?.deref().clone();
            let pos = *entity_ref.get::<Position>()?;
            fakephysics.add_velocity(0., -0.03, 0.);
            drop(entity_ref);
            fakephysics.move_entity(game, entity, *fakephysics.get_velocity())?;
            let entity_ref = game.ecs.entity(entity)?;
            let mut physics = entity_ref.get_mut::<Physics>()?;
            *physics = fakephysics;
            let velocity = physics.get_velocity_mut();
            velocity.x *= 0.;
            velocity.y *= 0.9800000190734863;
            velocity.z *= 0.;
            if pos.on_ground {
                velocity.y *= -0.5;
            }
        }
    }
    Ok(())
}