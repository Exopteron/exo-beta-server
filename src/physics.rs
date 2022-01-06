use std::mem;

use glam::Vec3;
use hecs::Entity;
use num_traits::Zero;

use crate::{game::{Game, Position}, ecs::{systems::SysResult, EntityRef}, aabb::{AABBSize, AABB}, protocol::packets::Face};

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
        //log::info!("Called velocity {:?}", self.velocity);
        if self.velocity.x + self.velocity.y + self.velocity.z == 0. {
            //log::info!("Returning");
            return Ok(());
        }
        let entityref = game.ecs.entity(entity)?;
        let aabb = entityref.get::<AABBSize>()?;
        let mut position = entityref.get_mut::<Position>()?;
        let world = game.worlds.get(&position.world).ok_or(anyhow::anyhow!("No world"))?;
        let real_aabb = aabb.get(&position);
        let mut doit = true;
        for (registry_type, state, pos) in world.get_possible_collisions(&*aabb, &*position) {
            if let Some(bounding_box) = registry_type.collision_box(state, pos) {
                let sweep_result = AABB::swept_aabb(real_aabb, bounding_box, self.velocity);
                if sweep_result.0 < 1. {
                    log::info!("Collision time: {}, Normals: {:?}, colliding with ID {} which is at {} in the world", sweep_result.0, sweep_result.1, state.b_type, pos);
                    let mut epic_box = real_aabb;
                    epic_box.minx += self.velocity.x as f64 * sweep_result.0;
                    epic_box.maxx += self.velocity.x as f64 * sweep_result.0;
                    epic_box.miny += self.velocity.y as f64 * sweep_result.0;
                    epic_box.maxy += self.velocity.y as f64 * sweep_result.0;
                    epic_box.minz += self.velocity.z as f64 * sweep_result.0;
                    epic_box.maxz += self.velocity.z as f64 * sweep_result.0;
                    let pos = epic_box.get_position(&*aabb, position.world);
                    doit = false;
                    *position = pos;
                    let error_margin = f64::EPSILON;
                    if (sweep_result.1.0 - -1.).abs() < error_margin || (sweep_result.1.0 - 1.).abs() < error_margin {
                        self.velocity.x = 0.;
                    }
                    if (sweep_result.1.2 - -1.).abs() < error_margin || (sweep_result.1.2 - 1.).abs() < error_margin {
                        self.velocity.z = 0.;
                    }
                    if (sweep_result.1.1 - 1.).abs() < error_margin ||  (sweep_result.1.1 - 1.).abs() < error_margin  {
                        self.velocity.y = 0.;
                    }
                    log::info!("Velocity vector: {:?}", self.velocity);
                }
            }
        }
        if doit {
            let x_offset = self.velocity.x * self.speed;
            self.velocity.x -= x_offset;
    
            let y_offset = self.velocity.y * self.speed;
            self.velocity.y -= y_offset;
    
            let z_offset = self.velocity.z * self.speed;
            self.velocity.z -= z_offset;
            position.x += x_offset as f64;
            position.y += y_offset as f64;
            position.z += z_offset as f64;
        }
        //log::info!("Position: {:?}", *position);
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