use crate::{
    item::item::{block::Block, BlockIdentifier},
    protocol::packets::Face, server::Server,
};

use super::{FluidBlock, FluidMaterial};

pub struct MovingLavaBlock(pub BlockIdentifier);
impl FluidBlock for MovingLavaBlock {
    fn id(&self) -> BlockIdentifier {
        self.0
    }
    fn is_same_material(id: u8) -> bool {
        Self::is_lava(id)
    }
    fn material() -> super::FluidMaterial {
        FluidMaterial::Lava
    }
    fn tick_rate() -> u128 {
        30
    }
    fn light_emittance(&self) -> u8 {
        1
    }

    fn opacity() -> u8 {
        15
    }
}
pub struct NotFlowingLavaBlock;
impl Block for NotFlowingLavaBlock {
    fn id(&self) -> BlockIdentifier {
        11
    }

    fn light_emittance(&self) -> u8 {
        1
    }

    fn opacity(&self) -> u8 {
        15
    }

    fn item_stack_size(&self) -> i8 {
        1
    }
    fn is_solid(&self) -> bool {
        false
    }
    fn added(&self, world: i32, game: &mut crate::game::Game, server: &mut crate::server::Server, position: crate::game::BlockPosition, state: crate::world::chunks::BlockState) {
        let _ = MovingLavaBlock::check_for_harden(world, server, game, position);
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
                scheduler.schedule_task(game.ticks + MovingLavaBlock::tick_rate(), move |game| {
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
            MovingLavaBlock::check_for_harden(world, &mut s, game, position)?;
        }
        Ok(())
    }
    fn can_place_over(&self) -> bool {
        true
    }
}
