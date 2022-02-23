use std::mem;

use glam::{Vec3, DVec3};
use hecs::Entity;
use num_traits::Zero;

use crate::{game::{Game, Position}, ecs::{systems::SysResult, EntityRef}, aabb::{AABBSize, AABB}, protocol::packets::Face};

#[derive(Default, Clone)]
pub struct Physics {
    modified: bool,
    velocity: DVec3,
    active: bool,
    speed: f64,
    step_height: f64,
}
impl Physics {
    pub fn new(active: bool, speed: f64, step_height: f64) -> Self {
        Self { velocity: DVec3::new(0., 0., 0.), active, speed, modified: false, step_height }
    }
    pub fn set_modified(&mut self, val: bool) {
        self.modified = val;
    }
    pub fn modified(&self) -> bool {
        self.modified
    }
    pub fn get_velocity(&self) -> &DVec3 {
        &self.velocity
    }
    pub fn get_velocity_mut(&mut self) -> &mut DVec3 {
        self.modified = true;
        &mut self.velocity
    }
    pub fn add_velocity(&mut self, x: f64, y: f64, z: f64) {
        self.modified = true;
        self.velocity.x += x;
        self.velocity.y += y;
        self.velocity.z += z;
    }
    pub fn move_entity(&mut self, game: &mut Game, entity: Entity, mut movement: DVec3) -> SysResult {
        let entityref = game.ecs.entity(entity)?;
        let aabb = entityref.get::<AABBSize>()?;
        let mut position = entityref.get_mut::<Position>()?;
        let world = game.worlds.get(&position.world).ok_or(anyhow::anyhow!("No world"))?;
        let mut real_aabb = aabb.get(&position);
        let ym = movement.y;
        let mut d6 = movement.x;
        let mut d7 = movement.y;
        let mut d8 = movement.z;
        let list = world.get_colliding_bbs(&*aabb, Some(real_aabb.add(movement.x as f64, movement.y as f64, movement.z as f64)), &*position);
        for item in list.iter() {
            movement.y = item.y_off(&real_aabb, movement.y as f64);   
        }
        // if movement.y > ym {
        //     log::info!("Greater {}", movement.y);
        //     let x = movement.y - ym.max(0.);
        //     log::info!("x {}", x);
        //     if x > self.step_height {
        //         movement.y = 0.;
        //     }
        // } else {
        //     log::info!("Less");
        // }
        real_aabb = real_aabb.offset(0., movement.y, 0.);
        let flag = position.on_ground;
        let flag1 = position.on_ground || d7 != movement.y && d7 < 0.;
        for item in list.iter() {
            movement.x = item.x_off(&real_aabb, movement.x);
        }
        real_aabb = real_aabb.offset(movement.x, 0., 0.);

        for item in list.iter() {
            movement.z = item.z_off(&real_aabb, movement.z);
        }
        real_aabb = real_aabb.offset(0., 0., movement.z);
        if self.active {
            let clone = *position;
            *position = real_aabb.get_position(&*aabb, position.world);
            position.yaw = clone.yaw;
            position.pitch = clone.pitch;
        }
        position.on_ground = d7 != movement.y && d7 < 0.;
        Ok(())
    }
    pub fn process(&mut self, game: &mut Game, entity: Entity) -> SysResult {
        self.move_entity(game, entity, self.velocity)?;
        self.velocity.x *= 0.99;
        self.velocity.y *= 0.99;
        self.velocity.z *= 0.99;
        return Ok(());
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
                    let pos = epic_box.get_position(&*aabb, position.world); // does not carry yaw/pitch
                    doit = false;
                    *position = pos;
                    self.velocity.x = 0.;
                    self.velocity.y = 0.;
                    self.velocity.z = 0.;
/*                     let error_margin = f64::EPSILON;
                    if (sweep_result.1.0 - -1.).abs() < error_margin || (sweep_result.1.0 - 1.).abs() < error_margin {
                        self.velocity.x = 0.;
                    }
                    if (sweep_result.1.2 - -1.).abs() < error_margin || (sweep_result.1.2 - 1.).abs() < error_margin {
                        self.velocity.z = 0.;
                    }
                    if (sweep_result.1.1 - 1.).abs() < error_margin ||  (sweep_result.1.1 - 1.).abs() < error_margin  {
                        self.velocity.y = 0.;
                    }
                    log::info!("Velocity vector: {:?}", self.velocity); */
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
        log::info!("Position: {:?}", *position);
        Ok(())
    }
    pub fn system(game: &mut Game) -> SysResult {
/*         let mut to_sim = Vec::new();
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
        } */
        Ok(())
    }
}