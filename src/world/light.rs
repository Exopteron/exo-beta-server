use std::{collections::VecDeque, sync::Arc};

use crate::{game::{Game, BlockPosition}, protocol::packets::Face, item::item::ItemRegistry};

pub struct LightPropagator {
    sky_light: bool,
    queue: VecDeque<(BlockPosition, u8)>
}
impl LightPropagator {
    pub fn new(sky_light: bool) -> Self {
        Self {
            sky_light,
            queue: VecDeque::new()
        }
    }
    pub fn increase_light(&mut self, world: i32, game: &mut Game, position: BlockPosition, value: u8) {
        if value > 15 {
            return;
        }
        if let Some(mut state) = game.block(position, world) {
            let light = match self.sky_light {
                true => state.b_skylight,
                false => state.b_light
            };
            if light < value || self.sky_light {
                self.queue.push_back((position, value));
                match self.sky_light {
                    true => state.b_skylight = value,
                    false => state.b_light = value,
                }
                game.set_block_nb(position, state, world, false, true);
                self.propagate(world, game);
            }
        }
    }
    fn propagate(&mut self, world: i32, game: &mut Game) {
        let mut registry = ItemRegistry::global();
        loop {
            if self.queue.len() > 0 {
                //log::info!("In queue");
                let (pos, light_value) = self.queue.pop_front().unwrap();
                for face in Face::all_faces() {
                    let neighbor = face.offset(pos);
                    if let Some(neighbor_state) = game.block(neighbor, world) {
                        let current_level;
                        current_level = match self.sky_light {
                            true => {
                                neighbor_state.b_skylight
                            }
                            false => {
                                neighbor_state.b_light
                            }
                        };
                        //log::info!("Neighbor on {:?} is {}", face, current_level);
                        if current_level >= (light_value - 1) {
                            continue;
                        }
                        let mut target_level = light_value - 1.max(match Self::is_opaque(neighbor_state.b_type, registry.clone()) {
                            true => 15,
                            false => 0,
                        });
                        if target_level > 15 {
                            target_level = 0;
                        }
                        if target_level > current_level {
                            if let Some(mut bstate) = game.block(neighbor, world) {
                                match self.sky_light {
                                    true => bstate.b_skylight = target_level,
                                    false => bstate.b_light = target_level
                                }
                                game.set_block_nb(neighbor, bstate, world, false, true);
                            }
                            self.queue.push_back((neighbor, target_level));
                        }
                    }
                }
            } else {
                break;
            }
        }
    }
    fn is_opaque(id: u8, registry: Arc<ItemRegistry>) -> bool {
        if let Some(block) = registry.get_block(id) {
            return block.opaque();
        }
        true
    }
}

pub fn propagate_light(world: i32, game: &mut Game, position: BlockPosition, mut light_level: u8, sky_light: bool) {
    //log::info!("Called");
    if game.is_solid_block(position, world) {
        log::info!("Solid");
        return;
    }
    light_level -= 1;
    if light_level == 0 {
        return;
    }
    if let Some(mut block) = game.block(position, world) {
        let light = match sky_light {
            true => block.b_skylight,
            false => block.b_light
        };
        if light_level <= 0 {
            log::info!("skylight? {} of {:?} is {}, higher than {}", sky_light, position, light, light_level);
            return;
        }
        if light >= light_level {
            return;
        }
        match sky_light {
            true => block.b_skylight = light,
            false => block.b_light = light
        }
        log::info!("Setting skylight? {} of {:?} to {}", sky_light, position, light);
        game.set_block(position, block, world);
        for face in Face::all_faces() {
            log::info!("Propagating to {:?} with a light level of {}", face, light_level);
            propagate_light(world, game, face.offset(position), light_level, sky_light);
        }
    } else {
        log::info!("No block");
    }
}