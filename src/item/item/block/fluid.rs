use crate::{
    game::{BlockPosition, Game},
    item::item::{BlockIdentifier, ItemRegistry},
    world::chunks::BlockState,
};
pub mod water;
use super::Block;

pub trait FluidBlock {
    fn id(&self) -> BlockIdentifier;
    fn flow_rate(&self) -> i32 {
        5
    }
    fn can_place_over(&self) -> bool {
        true
    }
    fn tick(
        &self,
        world: i32,
        game: &mut crate::game::Game,
        mut state: crate::world::chunks::BlockState,
        position: crate::game::BlockPosition,
        reschedule: &mut Option<u128>,
    ) {
        log::info!("Tick called");
        let mut var6 = self.get_flow_decay(world, game, position);
        let mut var7 = 1;
        let mut var10 = 0;
        let mut var8 = true;
        if var6 > 0 {
            let mut var9 = -100;
            let mut num_adjacent_sources = 0;
            let mut var12 = self.get_smallest_flow_decay(
                world,
                game,
                position.offset(-1, 0, 0),
                var9,
                &mut num_adjacent_sources,
            );
            var12 = self.get_smallest_flow_decay(
                world,
                game,
                position.offset(1, 0, 0),
                var12,
                &mut num_adjacent_sources,
            );
            var12 = self.get_smallest_flow_decay(
                world,
                game,
                position.offset(0, 0, -1),
                var12,
                &mut num_adjacent_sources,
            );
            var12 = self.get_smallest_flow_decay(
                world,
                game,
                position.offset(0, 0, 1),
                var12,
                &mut num_adjacent_sources,
            );
            var10 = var12 + var7;
            if var10 >= 8 || var12 < 0 {
                var10 = -1;
            }
            if self.get_flow_decay(world, game, position.offset(0, 1, 0)) >= 0 {
                let var11 = self.get_flow_decay(world, game, position.offset(0, 1, 0));
                if var11 >= 8 {
                    var10 = var11;
                } else {
                    var10 = var11 + 8;
                }
            }
            if num_adjacent_sources >= 2 {
                if game.is_solid_block(position.offset(0, -1, 0), world) {
                    var10 = 0;
                } else if self.is_water(game.block_id_at(position.offset(0, -1, 0), world))
                    && game.block_meta_at(position.offset(0, -1, 0), world) == 0
                {
                    var10 = 0;
                }
            }
            if var10 == var6 {
                if var8 {
                    self.update_flow(world, game, position);
                }
            } else {
                var6 = var10;
                if var10 < 0 {
                    game.set_block_nb(position, BlockState::air(), world, false, false);
                } else {
                    let bid = game.block_id_at(position, world);
                    game.set_block(position, BlockState::new(bid, var10 as u8), world);
                    // RESCHEDULE
                    *reschedule = Some(game.ticks + 5);
                    // END
                }
            }
        } else {
            self.update_flow(world, game, position);
        }
        if self.liquid_can_displace_block(world, game, position.offset(0, -1, 0)) {
            if var6 >= 8 {
                self.flow_into_block(world, game, position.offset(0, -1, 0), var6);
            } else {
                //game.set_block(position.offset(0, -1, 0), BlockState::new(self.id(), (var6 + 8) as u8), world);
                self.flow_into_block(world, game, position.offset(0, -1, 0), var6 + 8);
            }
        } else if var6 >= 0
            && (var6 == 0 || self.block_blocks_flow(world, game, position.offset(0, -1, 0)))
        {
            let var13 = self.get_optimal_flow_directions(world, game, position);
            var10 = var6 + var7;
            if var6 >= 8 {
                var10 = 1;
            }
            if var10 >= 8 {
                return;
            }
            if var13[0] {
                self.flow_into_block(world, game, position.offset(-1, 0, 0), var10);
            }
            if var13[1] {
                self.flow_into_block(world, game, position.offset(1, 0, 0), var10);
            }
            if var13[2] {
                self.flow_into_block(world, game, position.offset(0, 0, -1), var10);
            }
            if var13[3] {
                self.flow_into_block(world, game, position.offset(0, 0, 1), var10);
            }
        }
        return;
    }
    fn is_water(&self, id: u8) -> bool {
        id == 8 || id == 9
    }
    // SOLID
    fn flow_into_block(&self, world: i32, game: &mut Game, position: BlockPosition, par5: i32) {
        if self.liquid_can_displace_block(world, game, position) {
            let var6 = game.block_id_at(position, world);
            if var6 > 0 {}
            game.set_block_nb(position, BlockState::new(self.id(), par5 as u8), world, false, false);
        }
    }
    fn calculate_flow_cost(
        &self,
        world: i32,
        game: &mut Game,
        position: BlockPosition,
        par5: i32,
        par6: i32,
    ) -> i32 {
        let mut var7 = 1000;
        for var8 in 0..4 {
            if (var8 != 0 || par6 != 1)
                && (var8 != 1 || par6 != 0)
                && (var8 != 2 || par6 != 3)
                && (var8 != 3 || par6 != 2)
            {
                let mut var9 = position.x;
                let mut var11 = position.z;
                if var8 == 0 {
                    var9 = position.x - 1;
                }
                if var8 == 1 {
                    var9 += 1;
                }
                if var8 == 2 {
                    var11 = position.z - 1;
                }
                if var8 == 3 {
                    var11 += 1;
                }
                let position = BlockPosition::new(var9, position.y, var11);
                if !self.block_blocks_flow(world, game, position)
                    && !self.is_water(game.block_id_at(position, world))
                    || game.block_meta_at(position, world) != 0
                {
                    if !self.block_blocks_flow(world, game, position.offset(0, -1, 0)) {
                        return par5;
                    }
                    if par5 < 4 {
                        let var12 = self.calculate_flow_cost(world, game, position, par5 + 1, var8);
                        if var12 < var7 {
                            var7 = var12;
                        }
                    }
                }
            }
        }
        return var7;
    }
    fn get_optimal_flow_directions(
        &self,
        world: i32,
        game: &mut Game,
        position: BlockPosition,
    ) -> [bool; 4] {
        let mut var6 = 0;
        let mut flow_cost = [0; 4];
        for var5 in 0i32..4i32 {
            flow_cost[var5 as usize] = 1000;
            let mut var8 = position.z;
            if var5 == 0 {
                var6 = position.x - 1;
            }
            if var5 == 1 {
                var6 += 1;
            }
            if var5 == 2 {
                var8 = position.z - 1;
            }
            if var5 == 3 {
                var8 += 1;
            }
            let position = BlockPosition::new(var6, position.y, var8);
            if !self.block_blocks_flow(world, game, position)
                && (!self.is_water(game.block_id_at(position, world))
                    || game.block_meta_at(position, world) != 0)
            {
                if self.block_blocks_flow(world, game, position.offset(0, -1, 0)) {
                    flow_cost[var5 as usize] =
                        self.calculate_flow_cost(world, game, position, 1, var5);
                } else {
                    flow_cost[var5 as usize] = 0;
                }
            }
        }
        let mut var5 = flow_cost[0];
        for var6 in 1..4 {
            if flow_cost[var6] < var5 {
                var5 = flow_cost[var6];
            }
        }
        let mut is_optimal_flow_direction = [false; 4];
        for var6 in 0..4 {
            is_optimal_flow_direction[var6] = flow_cost[var6] == var5;
        }
        is_optimal_flow_direction
    }
    /// SOLID
    fn block_blocks_flow(&self, world: i32, game: &mut Game, position: BlockPosition) -> bool {
        let var5 = game.block_id_at(position, world);
        if var5 == 0 {
            return false;
        }
        game.is_solid_block(position, world)
    }
    /// SOLID
    fn liquid_can_displace_block(
        &self,
        world: i32,
        game: &mut Game,
        position: BlockPosition,
    ) -> bool {
        let id = game.block_id_at(position, world);
        if self.is_water(id) {
            return false;
        }
        !self.block_blocks_flow(world, game, position)
    }
    /// SOLID
    fn update_flow(&self, world: i32, game: &mut Game, position: BlockPosition) {
        if let Some(mut blockstate) = game.block(position, world) {
            blockstate.b_type += 1;
            //log::info!("Setting {:?} to {}", position, blockstate.b_type);
            game.set_block_nb(position, blockstate, world, false, false);
        }
    }
    /// SOLID
    fn get_flow_decay(&self, world: i32, game: &mut Game, position: BlockPosition) -> i32 {
        if let Some(block) = game.block(position, world) {
            if self.is_water(block.b_type) {
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
        let mut var6 = self.get_flow_decay(world, game, position);
        if var6 < 0 {
            return current_smallest_flow_decay;
        } else {
            if var6 == 0 {
                *numadjsource += 1;
            }
            if var6 >= 8 {
                var6 = 0;
            }
            if current_smallest_flow_decay >= 0 && var6 >= current_smallest_flow_decay {
                return current_smallest_flow_decay;
            } else {
                return var6;
            }
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

    fn item_stack_size(&self) -> i8 {
        1
    }

    fn is_solid(&self) -> bool {
        false
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
    fn added(
        &self,
        world: i32,
        game: &mut crate::game::Game,
        server: &mut crate::server::Server,
        position: crate::game::BlockPosition,
        state: crate::world::chunks::BlockState,
    ) {
        let s = game.scheduler.clone();
        let mut scheduler = s.borrow_mut();
        if game.block_id_at(position, world) == 8 {
            scheduler.schedule_task(game.ticks + 5, move |game| {
                if let Some(block) = game.block(position, world) {
                    if let Ok(b) = block.registry_type() {
                        let mut reschedule = None;
                        b.upd_tick(world, game, block, position, &mut reschedule);
                        return reschedule;
                    }
                }
                None
            });
        }
    }
}
