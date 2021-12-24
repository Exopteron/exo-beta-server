use crate::{item::{item::{block::{Block, ActionResult}, ItemRegistry, Item, ItemIdentifier}, stack::ItemStackType, window::Window}, protocol::packets::{Face, SoundEffectKind}, events::block_interact::BlockPlacementEvent, world::chunks::BlockState, ecs::{systems::SysResult, entities::player::Gamemode}, game::{Position, BlockPosition}, network::ids::NetworkID, server::Server};

pub struct WaterBucketItem;
impl Item for WaterBucketItem {
    fn id(&self) -> crate::item::item::ItemIdentifier {
        326
    }

    fn stack_size(&self) -> i8 {
        1
    }

    fn durability(&self) -> Option<i16> {
        None
    }
    fn on_use(&self, game: &mut crate::game::Game, server: &mut Server, item_slot: usize, user: hecs::Entity, target: Option<crate::item::item::BlockUseTarget>) -> SysResult {
        // TODO reduce in gms
        if let Some(target) = target {
            let block_pos = target.face.offset(target.position);
            game.set_block(block_pos, BlockState::new(8, 0), target.world);
        }
        Ok(())
    }
}