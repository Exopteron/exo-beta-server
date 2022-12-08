use hecs::Entity;
use rand::Rng;

use crate::{
    ecs::{entities::item::ItemEntityBuilder, systems::SysResult},
    game::{BlockPosition, Game, Position},
    item::{
        inventory_slot::InventorySlot,
        item::{BlockIdentifier, ItemRegistry},
    },
    protocol::packets::{server::SoundEffect, Face, SoundEffectKind},
    server::Server,
    world::chunks::BlockState,
};
pub mod lava;
pub mod water;
use super::Block;

macro_rules! meta_is_falling {
    ($v:expr) => {
        ($v >= 8)
    };
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FluidMaterial {
    Lava,
    Water,
}

pub trait FluidBlock {
    fn id(&self) -> BlockIdentifier;
    fn flow_rate(&self) -> i32 {
        5
    }
    fn light_emittance(&self) -> u8 {
        1
    }
    fn on_collide(&self, game: &mut Game, position: BlockPosition, state: BlockState, entity: Entity) -> SysResult { Ok(()) }
    fn material() -> FluidMaterial;
    fn opacity() -> u8;
    fn check_for_harden(
        world: i32,
        server: &mut Server,
        game: &mut Game,
        position: BlockPosition,
    ) -> SysResult {
        if !Self::is_same_material(game.block_id_at(position)) {
            return Ok(());
        }
        if Self::material() == FluidMaterial::Lava {
            let mut flag = false;
            for face in Face::all_faces() {
                let pos = face.offset(position);
                if Self::is_water(game.block_id_at(pos)) {
                    flag = true;
                    break;
                }
            }
            if flag {
                let meta = game.block_meta_at(position, position.world);
                if meta == 0 {
                    game.set_block_nb(
                        position,
                        BlockState::new(49, 0),
                        position.world,
                        false,
                        false,
                        true,
                    );
                } else {
                    game.set_block_nb(
                        position,
                        BlockState::new(4, 0),
                        position.world,
                        false,
                        false,
                        true,
                    );
                }
                game.schedule_next_tick(move |game| {
                    if meta == 0 {
                        game.set_block_nb(
                            position,
                            BlockState::new(49, 0),
                            position.world,
                            false,
                            false,
                            true,
                        );
                    } else {
                        game.set_block_nb(
                            position,
                            BlockState::new(4, 0),
                            position.world,
                            false,
                            false,
                            true,
                        );
                    }
                    None
                });
                server.broadcast_effect(SoundEffectKind::Extinguish, position, position.world, 0);
            }
        }
        Ok(())
    }

    fn tick_rate() -> u128;
    fn tick(
        &self,
        world: i32,
        game: &mut crate::game::Game,
        mut state: crate::world::chunks::BlockState,
        position: crate::game::BlockPosition,
        reschedule: &mut Option<u128>,
    ) {
        if !Self::is_same_material(state.b_type) {
            return;
        }
        let s = game.scheduler.clone();
        //log::info!("Tick called");
        let mut our_flow_decay = self.get_flow_decay(world, game, position);
        let mut flow_tick_rate = 1;
        if Self::material() == FluidMaterial::Lava {
            flow_tick_rate = 2;
        }
        let mut flag = true;

        // if we are not a source block
        if our_flow_decay > 0 {
            let mut smallest_flow_decay = -100;
            let mut num_adjacent_sources = 0;
            smallest_flow_decay = self.get_smallest_flow_decay(
                world,
                game,
                position.offset(-1, 0, 0),
                smallest_flow_decay,
                &mut num_adjacent_sources,
            );
            smallest_flow_decay = self.get_smallest_flow_decay(
                world,
                game,
                position.offset(1, 0, 0),
                smallest_flow_decay,
                &mut num_adjacent_sources,
            );
            smallest_flow_decay = self.get_smallest_flow_decay(
                world,
                game,
                position.offset(0, 0, -1),
                smallest_flow_decay,
                &mut num_adjacent_sources,
            );
            smallest_flow_decay = self.get_smallest_flow_decay(
                world,
                game,
                position.offset(0, 0, 1),
                smallest_flow_decay,
                &mut num_adjacent_sources,
            );

            // smallest flow decay is now the smallest metadata of surrounding water blocks
            // or -100 with no surrounding water blocks
            let mut new_flow_decay_to_set = smallest_flow_decay + flow_tick_rate;
            if meta_is_falling!(new_flow_decay_to_set) || smallest_flow_decay < 0 {
                new_flow_decay_to_set = -1;
            }

            // if the flow decay of the block above us is greater than or equal to 0 (a water block)
            if self.get_flow_decay(world, game, position.offset(0, 1, 0)) >= 0 {
                // get its flow decay
                let block_above_flow_decay =
                    self.get_flow_decay(world, game, position.offset(0, 1, 0));
                if meta_is_falling!(block_above_flow_decay) {
                    // if it is a full non source block
                    new_flow_decay_to_set = block_above_flow_decay;
                } else {
                    // otherwise make it a full non source block with meta by adding 8
                    new_flow_decay_to_set = 8;
                }
            }
            // if there are more than two adjacent sources
            if num_adjacent_sources >= 2 && Self::material() == FluidMaterial::Water {
                // if the block below us is solid
                if game.is_solid_block(position.offset(0, -1, 0)) {
                    // set the new flow decay to zero
                    new_flow_decay_to_set = 0;
                } else if Self::is_water(game.block_id_at(position.offset(0, -1, 0)))
                    && game.block_meta_at(position.offset(0, -1, 0), world) == 0
                {
                    // otherwise if the block below us is a water source block with no special meta
                    // set it to zero as well
                    new_flow_decay_to_set = 0;
                }
            }
            if Self::material() == FluidMaterial::Lava
                && our_flow_decay < 8
                && new_flow_decay_to_set < 8
                && new_flow_decay_to_set > our_flow_decay
                && game.rng.gen_range(0..4) != 0
            {
                new_flow_decay_to_set = our_flow_decay;
                flag = false;
            }

            // if the new flow decay is not equal to our current flow decay
            if new_flow_decay_to_set != our_flow_decay {
                our_flow_decay = new_flow_decay_to_set;

                if our_flow_decay < 0 {
                    // if it is less than zero, then we are probably surrounded by air blocks
                    // or the condition above triggered
                    game.set_block_nb(position, BlockState::air(), world, true, false, false);
                } else {
                    // otherwise set ourselves to the new flow decay

                    game.set_block_nb(
                        position,
                        BlockState::new(self.id(), our_flow_decay as u8),
                        world,
                        true,
                        false,
                        false,
                    );
                    game.set_block_nb(
                        position,
                        BlockState::new(self.id(), our_flow_decay as u8),
                        world,
                        true,
                        false,
                        false,
                    );

                    let id = self.id();
                    // schedule a block update on ourself in 5 ticks
                    s.borrow_mut()
                        .schedule_task(game.ticks + Self::tick_rate(), move |game| {
                            if let Some(block) = game.block(position, world) {
                                if block.b_type == id {
                                    if let Ok(b) = block.registry_type() {
                                        let mut reschedule = None;
                                        b.upd_tick(world, game, block, position, &mut reschedule);
                                        return reschedule;
                                    }
                                }
                            }
                            None
                        });
                }
            } else if flag {
                // else make ourselves non-flowing
                self.update_flow(world, game, position);
            }
        } else {
            // otherwise make ourselves non-flowing
            self.update_flow(world, game, position);
        }
        // if the block below us can be displaced by liquid
        if self.liquid_can_displace_block(world, game, position.offset(0, -1, 0)) {
            if meta_is_falling!(our_flow_decay) {
                // if we are a falling water then make the block below us the same
                game.set_block_nb(
                    position.offset(0, -1, 0),
                    BlockState::new(self.id(), our_flow_decay as u8),
                    world,
                    true,
                    false,
                    false,
                );
            } else {
                // otherwise if we are a source block or a non-full water then make the block below us
                // falling
                game.set_block_nb(
                    position.offset(0, -1, 0),
                    BlockState::new(self.id(), 0x8),
                    world,
                    true,
                    false,
                    false,
                );
            }
        } else if our_flow_decay >= 0
            && (our_flow_decay == 0
                || self.block_blocks_flow(world, game, position.offset(0, -1, 0)))
        {
            // otherwise if our flow decay is greater than or equal to zero
            // AND (we are a source block OR the block below us blocks water flow)

            // get the optimal flow directions for all cardinal directions

            let var13 = self.get_optimal_flow_directions(world, game, position);

            let mut new_decay_here = our_flow_decay + flow_tick_rate;
            // if our flow decay represents a falling water block
            if meta_is_falling!(our_flow_decay) {
                // make the new decay the largest non-full water block
                new_decay_here = 1;
            }
            if meta_is_falling!(new_decay_here) {
                // if our new decay is a falling water block, return without flowing
                return;
            }
            // flow into all the optimal directions
            if var13[0] {
                self.flow_into_block(world, game, position.offset(-1, 0, 0), new_decay_here);
            }
            if var13[1] {
                self.flow_into_block(world, game, position.offset(1, 0, 0), new_decay_here);
            }
            if var13[2] {
                self.flow_into_block(world, game, position.offset(0, 0, -1), new_decay_here);
            }
            if var13[3] {
                self.flow_into_block(world, game, position.offset(0, 0, 1), new_decay_here);
            }
        }
        return;
    }
    fn is_water(id: u8) -> bool {
        id == 8 || id == 9
    }
    fn is_lava(id: u8) -> bool {
        id == 10 || id == 11
    }
    fn is_same_material(id: u8) -> bool;
    // SOLID
    fn flow_into_block(&self, world: i32, game: &mut Game, position: BlockPosition, par5: i32) {
        if self.liquid_can_displace_block(world, game, position) {
            let var6 = game.block_id_at(position);

            let state = game.block_meta_at(position, position.world);
            if var6 > 0 {
                if let Some(item) = ItemRegistry::global().get_block(var6) {
                    let dropped_items =
                        item.dropped_items(BlockState::new(var6, state), InventorySlot::Empty);
                    for item in dropped_items {
                        let entity = ItemEntityBuilder::build(game, Position::from(position), item, 5);
                        game.spawn_entity(entity);
                    }
                }
            }
            game.set_block_nb(
                position,
                BlockState::new(self.id(), par5 as u8),
                world,
                true,
                false,
                false,
            );
        }
    }
    fn calculate_flow_cost(
        &self,
        world: i32,
        game: &mut Game,
        position: BlockPosition,
        num_repetitions: i32,
        i1: i32,
    ) -> i32 {
        let mut current_flow_cost = 1000;
        for iteration_count in 0..4 {
            if iteration_count == 0 && i1 == 1
                || iteration_count == 1 && i1 == 0
                || iteration_count == 2 && i1 == 3
                || iteration_count == 3 && i1 == 2
            {
                continue;
            }

            let mut l1 = position.x;
            let mut i2 = position.y;
            let mut j2 = position.z;

            if (iteration_count == 0) {
                l1 -= 1;
            }
            if (iteration_count == 1) {
                l1 += 1;
            }
            if (iteration_count == 2) {
                j2 -= 1;
            }
            if (iteration_count == 3) {
                j2 += 1;
            }

            // on first iteration, [X - 1, Y, Z]
            // on second iteration, [X + 1, Y, Z]
            // on third iteration, [X, Y, Z - 1]
            // on fourth iteration, [X, Y, Z + 1]

            let position = BlockPosition::new(l1, i2, j2, position.world);

            if self.block_blocks_flow(world, game, position)
                || Self::is_same_material(game.block_id_at(position))
                    && game.block_meta_at(position, world) == 0
            {
                // if this block blocks water flow or is a water source block
                // continue, leaving the flow cost for the origin at 1000
                continue;
            }

            if !self.block_blocks_flow(world, game, position.offset(0, -1, 0)) {
                // if the block below us does not block water flow, return the amount of repetitions we had
                // to do to find this available space
                return num_repetitions;
            }

            if num_repetitions >= 4 {
                // if the number of repetitions is greater than 4, continue
                continue;
            }

            // recursively calculate the flow cost for this direction of the origin block
            let k2 = self.calculate_flow_cost(
                world,
                game,
                position,
                num_repetitions + 1,
                iteration_count,
            );
            // if it is lower than the current flow cost
            if k2 < current_flow_cost {
                // assign it to the current flow cost
                current_flow_cost = k2;
            }
        }
        current_flow_cost
    }
    fn get_optimal_flow_directions(
        &self,
        world: i32,
        game: &mut Game,
        position: BlockPosition,
    ) -> [bool; 4] {
        let mut var6 = 0;
        let mut flow_cost = [0; 4];

        // for all 4 cardinal directions
        for l in 0i32..4i32 {
            // the cost is 1000 by default
            flow_cost[l as usize] = 1000;

            let mut j1 = position.x;
            let mut i2 = position.y;
            let mut j2 = position.z;
            let mut var8 = position.z;
            if l == 0 {
                j1 -= 1;
            }
            if l == 1 {
                j1 += 1;
            }
            if l == 2 {
                j2 -= 1;
            }
            if l == 3 {
                j2 += 1;
            }

            // on first iteration, [X - 1, Y, Z]
            // on second iteration, [X + 1, Y, Z]
            // on third iteration, [X, Y, Z - 1]
            // on fourth iteration, [X, Y, Z + 1]

            let position = BlockPosition::new(j1, i2, j2, world);

            if self.block_blocks_flow(world, game, position)
                || Self::is_same_material(game.block_id_at(position))
                    && game.block_meta_at(position, world) == 0
            {
                // if this block blocks water flow or it is a water source block, continue
                // leaving the flow cost at 1000
                continue;
            }

            // if the block below us does not block water flow
            if !self.block_blocks_flow(world, game, position.offset(0, -1, 0)) {
                // set the flow cost for this direction to 0
                flow_cost[l as usize] = 0;
            } else {
                // otherwise, calculate the flow cost for this offshoot of the origin
                flow_cost[l as usize] = self.calculate_flow_cost(world, game, position, 1, l);
            }
        }

        // find the lowest flow cost among all of the
        // directions
        let mut lowest_flow_cost = flow_cost[0];
        for k1 in 1..4 {
            if flow_cost[k1] < lowest_flow_cost {
                lowest_flow_cost = flow_cost[k1];
            }
        }

        // say if each direction is optimal by seeing if its flow cost is the lowest within all of the options
        let mut is_optimal_flow_direction = [false; 4];
        for l1 in 0..4 {
            is_optimal_flow_direction[l1] = flow_cost[l1] == lowest_flow_cost;
        }
        is_optimal_flow_direction
    }
    /// SOLID
    fn block_blocks_flow(&self, world: i32, game: &mut Game, position: BlockPosition) -> bool {
        let var5 = game.block_id_at(position);
        if var5 == 0 {
            return false;
        }
        game.is_solid_block(position)
    }
    /// SOLID
    fn liquid_can_displace_block(
        &self,
        world: i32,
        game: &mut Game,
        position: BlockPosition,
    ) -> bool {
        let id = game.block_id_at(position);
        if Self::is_same_material(id) || Self::is_lava(id) {
            return false;
        }
        !self.block_blocks_flow(world, game, position)
    }
    /// SOLID
    fn update_flow(&self, world: i32, game: &mut Game, position: BlockPosition) {
        if let Some(mut blockstate) = game.block(position, world) {
            blockstate.b_type += 1;
            //log::info!("Setting {:?} to {}", position, blockstate.b_type);
            game.set_block_nb(position, blockstate, world, false, false, false);
            // let id = self.id();
            // game.schedule_next_tick(move |game| {
            //     if let Some(block) = game.block(position, world) {
            //         if block.b_type == id {
            //             if let Ok(b) = block.registry_type() {
            //                 let mut reschedule = None;
            //                 b.upd_tick(world, game, block, position, &mut reschedule);
            //                 return reschedule;
            //             }
            //         }
            //     }
            //     None
            // });
        }
    }
    /// SOLID
    fn get_flow_decay(&self, world: i32, game: &mut Game, position: BlockPosition) -> i32 {
        if let Some(block) = game.block(position, world) {
            if Self::is_same_material(block.b_type) {
                return block.b_metadata as i32;
            }
        }
        -1
    }
    /// SOLID
    fn get_smallest_flow_decay(
        &self,
        world: i32,
        game: &mut Game,
        position: BlockPosition,
        current_smallest_flow_decay: i32,
        numadjsource: &mut i32,
    ) -> i32 {
        let mut new_flow_decay = self.get_flow_decay(world, game, position);
        if new_flow_decay < 0 {
            // if it is not a water block
            return current_smallest_flow_decay;
        }
        if new_flow_decay == 0 {
            *numadjsource += 1;
        }
        if meta_is_falling!(new_flow_decay) {
            // metas greater than 8 are all falling blocks
            new_flow_decay = 0;
        }
        // if the new flow decay is less than the current, return it
        // else return the current
        if current_smallest_flow_decay >= 0 && new_flow_decay >= current_smallest_flow_decay {
            current_smallest_flow_decay
        } else {
            new_flow_decay
        }
    }
}
impl<T> Block for T
where
    T: FluidBlock,
{
    fn id(&self) -> crate::item::item::BlockIdentifier {
        FluidBlock::id(self)
    }

    fn passable(&self) -> bool {
        true
    }
    
    fn on_collide(&self, game: &mut Game, position: BlockPosition, state: BlockState, entity: Entity) -> SysResult {
        FluidBlock::on_collide(self, game, position, state, entity)
    }
    fn light_emittance(&self) -> u8 {
        FluidBlock::light_emittance(self)
    }

    fn item_stack_size(&self) -> i8 {
        1
    }

    fn can_place_over(&self) -> bool {
        true
    }

    fn is_solid(&self) -> bool {
        false
    }
    fn opacity(&self) -> u8 {
        <Self as FluidBlock>::opacity()
    }

    fn upd_tick(
        &self,
        world: i32,
        game: &mut crate::game::Game,
        state: crate::world::chunks::BlockState,
        position: crate::game::BlockPosition,
        reschedule: &mut Option<u128>,
    ) {
        FluidBlock::tick(self, world, game, state, position, reschedule)
    }

    fn neighbor_update(
        &self,
        world: i32,
        game: &mut Game,
        position: BlockPosition,
        state: BlockState,
        offset: Face,
        neighbor_state: BlockState,
    ) -> SysResult {
        let obj = game.objects.clone();
        let mut s = obj.get_mut::<Server>()?;
        Self::check_for_harden(world, &mut *s, game, position)?;
        Ok(())
    }

    fn added(
        &self,
        world: i32,
        game: &mut crate::game::Game,
        server: &mut crate::server::Server,
        position: crate::game::BlockPosition,
        state: crate::world::chunks::BlockState,
    ) {
        let _ = Self::check_for_harden(world, server, game, position);
        //println!("ADDED");
        let s = game.scheduler.clone();

        let id = self.id();
        if game.block_id_at(position) == self.id() {
            s.borrow_mut()
                .schedule_task(game.ticks + Self::tick_rate(), move |game| {
                    if let Some(block) = game.block(position, world) {
                        if block.b_type == id {
                            if let Ok(b) = block.registry_type() {
                                let mut reschedule = None;
                                b.upd_tick(world, game, block, position, &mut reschedule);
                                return reschedule;
                            }
                        }
                    }
                    None
                });
        }
    }
}
