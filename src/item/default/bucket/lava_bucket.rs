use parking_lot::MutexGuard;

use crate::{item::{item::{block::{Block, ActionResult}, ItemRegistry, Item, ItemIdentifier}, stack::{ItemStackType, ItemStack}, window::Window, inventory_slot::InventorySlot}, protocol::packets::{Face, SoundEffectKind}, events::block_interact::BlockPlacementEvent, world::chunks::BlockState, ecs::{systems::SysResult, entities::player::Gamemode}, game::{Position, BlockPosition}, network::ids::NetworkID, server::Server};

pub struct LavaBucketItem;
impl Item for LavaBucketItem {
    fn id(&self) -> crate::item::item::ItemIdentifier {
        327
    }

    fn stack_size(&self) -> i8 {
        1
    }

    fn durability(&self) -> Option<i16> {
        None
    }
    fn on_use(&self, game: &mut crate::game::Game, server: &mut Server, mut item: MutexGuard<InventorySlot>, slot: usize, user: hecs::Entity, target: Option<crate::item::item::BlockUseTarget>) -> SysResult {
        // TODO reduce in gms
        if let Some(target) = target {
            let block_pos = target.face.offset(target.position);
            game.set_block(block_pos, BlockState::new(10, 0), target.world);
            if let Ok(g) = game.ecs.get::<Gamemode>(user) {
                if *g != Gamemode::Creative {
                    *item = InventorySlot::Filled(ItemStack::new(325, 1, 0));
                }
            }
        }
        Ok(())
    }
}