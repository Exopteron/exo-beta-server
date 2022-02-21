use hecs::Entity;

use crate::{
    ecs::{entities::player::Chatbox, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{stack::{ItemStackType, ItemStack}, item::block::{ActionResult, AtomicRegistryBlock, NonBoxedRegBlock}},
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState, network::ids::NetworkID,
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};
pub struct GlassBlock;

impl Block for GlassBlock {
    fn dropped_items(&self, state: BlockState, held_item: crate::item::inventory_slot::InventorySlot) -> Vec<ItemStack> {
        Vec::new()
    }
    fn slipperiness(&self) -> f64 {
        5.
    }
    fn id(&self) -> BlockIdentifier {
        20
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn is_solid(&self) -> bool {
        false
    }
    fn opaque(&self) -> bool {
        false
    }
}