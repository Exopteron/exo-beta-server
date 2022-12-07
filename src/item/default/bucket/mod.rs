pub mod water_bucket;
pub mod lava_bucket;

use parking_lot::MutexGuard;

use crate::{item::{item::{block::{Block, ActionResult, fluid::{water::MovingWaterBlock, FluidBlock, lava::MovingLavaBlock}}, ItemRegistry, Item, ItemIdentifier}, stack::{ItemStackType, ItemStack}, window::Window, inventory_slot::InventorySlot}, protocol::packets::{Face, SoundEffectKind}, events::block_interact::BlockPlacementEvent, world::chunks::BlockState, ecs::{systems::SysResult, entities::player::Gamemode}, game::{Position, BlockPosition}, network::ids::NetworkID, server::Server};

pub struct BucketItem;
impl Item for BucketItem {
    fn id(&self) -> crate::item::item::ItemIdentifier {
        325
    }

    fn stack_size(&self) -> i8 {
        1
    }

    fn durability(&self) -> Option<i16> {
        None
    }
    fn on_use(&self, game: &mut crate::game::Game, server: &mut Server, mut item: MutexGuard<InventorySlot>, slot: usize, user: hecs::Entity, target: Option<crate::item::item::BlockUseTarget>) -> SysResult {
        // // TODO reduce in gms
        if let Some(target) = target {
            let block_pos = target.face.offset(target.position);
            if let Some(state) = game.block(block_pos, block_pos.world) {
                if MovingWaterBlock::is_water(state.b_type) && state.b_metadata == 0 {
                    game.set_block(block_pos, BlockState::air(), block_pos.world);
                    *item = InventorySlot::Filled(ItemStack::new(326, 1, 0));
                } else if MovingLavaBlock::is_lava(state.b_type) && state.b_metadata == 0 {
                    game.set_block(block_pos, BlockState::air(), block_pos.world);
                    *item = InventorySlot::Filled(ItemStack::new(327, 1, 0));
                }
            }
            
        }
        Ok(())
    }
}