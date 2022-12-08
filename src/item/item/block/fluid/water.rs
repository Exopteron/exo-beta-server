use crate::{
    item::item::{block::Block, BlockIdentifier},
    protocol::packets::Face, game::{Game, BlockPosition}, ecs::systems::SysResult, server::Server,
};

use super::{FluidBlock, FluidMaterial};

pub struct MovingWaterBlock(pub BlockIdentifier);
impl FluidBlock for MovingWaterBlock {
    fn id(&self) -> BlockIdentifier {
        self.0
    }
    fn is_same_material(id: u8) -> bool {
        Self::is_water(id)
    }
    fn material() -> super::FluidMaterial {
        FluidMaterial::Water
    }

    fn tick_rate() -> u128 {
        5
    }
    fn opacity() -> u8 {
        3
    }
}
pub struct NotFlowingWaterBlock;

impl MovingWaterBlock {

}
impl Block for NotFlowingWaterBlock {
    fn id(&self) -> BlockIdentifier {
        9
    }

    fn passable(&self) -> bool {
        true
    }
    fn opacity(&self) -> u8 {
        3
    }

    fn item_stack_size(&self) -> i8 {
        1
    }
    fn is_solid(&self) -> bool {
        false
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
            if game.block_id_at(position) == self.id() {
                state.b_type -= 1;
                let id = state.b_type;
                game.set_block_nb(position, state, world, false, false, true);
                let s = game.scheduler.clone();
                let mut scheduler = s.borrow_mut();
                scheduler.schedule_task(game.ticks + MovingWaterBlock::tick_rate(), move |game| {
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
        {
            let obj = game.objects.clone();
            let mut s = obj.get_mut::<Server>()?;
            MovingWaterBlock::check_for_harden(world, &mut *s, game, position)?;
        }
        Ok(())
    }
    fn added(&self, world: i32, game: &mut Game, server: &mut crate::server::Server, position: BlockPosition, state: crate::world::chunks::BlockState) {
        let _ = MovingWaterBlock::check_for_harden(world, server, game, position);
    }
    fn can_place_over(&self) -> bool {
        true
    }
}
