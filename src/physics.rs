use std::mem;

use glam::Vec3;
use hecs::Entity;

use crate::{game::{Game, Position}, ecs::{systems::SysResult, EntityRef}, aabb::{AABBSize, AABB}};

#[derive(Default)]
pub struct Physics {
    velocity: Vec3,
    active: bool,
    speed: f32,
}
impl Physics {
    pub fn new(active: bool, speed: f32) -> Self {
        Self { velocity: Vec3::new(0., 0., 0.), active, speed }
    }
    pub fn add_velocity(&mut self, x: f32, y: f32, z: f32) {
        self.velocity.x += x;
        self.velocity.y += y;
        self.velocity.z += z;
    }
    pub fn process(&mut self, game: &mut Game, entity: Entity) -> SysResult {
        log::info!("Called velocity {:?}", self.velocity);
        if self.velocity.x + self.velocity.y + self.velocity.z == 0. {
            log::info!("Returning");
            return Ok(());
        }
        let entityref = game.ecs.entity(entity)?;
        let aabb = entityref.get::<AABBSize>()?;
        let mut position = entityref.get_mut::<Position>()?;
        let world = game.worlds.get(&position.world).ok_or(anyhow::anyhow!("No world"))?;
        for (_, _, _, list) in world.get_collisions_extra(&*aabb, &*position) {
            for collision in list {
                log::info!("Collision: {:?}", collision);
                match collision {
                    crate::protocol::packets::Face::Invalid => (),
                    crate::protocol::packets::Face::NegativeY => self.velocity.y = self.velocity.y.max(0.),
                    crate::protocol::packets::Face::PositiveY => self.velocity.y = self.velocity.y.min(0.),
                    crate::protocol::packets::Face::NegativeZ => self.velocity.z = self.velocity.z.max(0.),
                    crate::protocol::packets::Face::PositiveZ => self.velocity.z = self.velocity.z.min(0.),
                    crate::protocol::packets::Face::NegativeX => self.velocity.x = self.velocity.x.max(0.),
                    crate::protocol::packets::Face::PositiveX => self.velocity.x = self.velocity.x.min(0.),
                };
            }
        }
        let x_offset = self.velocity.x * self.speed;
        self.velocity.x -= x_offset;

        let y_offset = self.velocity.y * self.speed;
        self.velocity.y -= y_offset;

        let z_offset = self.velocity.z * self.speed;
        self.velocity.z -= z_offset;
        position.x += x_offset as f64;
        position.y += y_offset as f64;
        position.z += z_offset as f64;
        log::info!("Position: {}", *position);
        Ok(())
    }
    pub fn system(game: &mut Game) -> SysResult {
        let mut to_sim = Vec::new();
        for (entity, _) in game.ecs.query::<&Physics>().iter() {
            to_sim.push(entity);
        }
        for entity in to_sim {
            let entityref = game.ecs.entity(entity)?;
            let mut physics = entityref.get_mut::<Physics>()?;
            let mut physics_cl = mem::take(&mut *physics);
            drop(entityref);
            drop(physics);
            physics_cl.process(game, entity)?;
            let entityref = game.ecs.entity(entity)?;
            let mut physics_ref = entityref.get_mut::<Physics>()?;
            *physics_ref = physics_cl;
        }
        Ok(())
    }
}