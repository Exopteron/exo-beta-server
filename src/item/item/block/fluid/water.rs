use crate::{
    item::item::{block::Block, BlockIdentifier},
    protocol::packets::Face,
};

use super::FluidBlock;

pub struct MovingWaterBlock(pub BlockIdentifier);
impl FluidBlock for MovingWaterBlock {
    fn id(&self) -> BlockIdentifier {
        self.0
    }
}
pub struct NotFlowingWaterBlock;
impl Block for NotFlowingWaterBlock {
    fn id(&self) -> BlockIdentifier {
        9
    }

    fn item_stack_size(&self) -> i8 {
        1
    }

    fn neighbor_update(
        &self,
        world: i32,
        game: &mut crate::game::Game,
        position: crate::game::BlockPosition,
        mut state: crate::world::chunks::BlockState,
        offset: crate::protocol::packets::Face,
        neighbor_state: crate::world::chunks::BlockState,
    ) -> crate::ecs::systems::SysResult {
        if !matches!(offset, Face::Invalid) {
            let neighbor = game.block_id_at(offset.offset(position), world);
            if game.block_id_at(position, world) == self.id() {
                state.b_type -= 1;
                game.set_block(position, state, world);
                let s = game.scheduler.clone();
                let mut scheduler = s.borrow_mut();
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
        Ok(())
    }
    fn can_place_over(&self) -> bool {
        true
    }
}
