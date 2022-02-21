use hecs::Entity;
use rand::Rng;

use crate::{
    ecs::{entities::player::Chatbox, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{stack::{ItemStackType, ItemStack}, item::block::{ActionResult, AtomicRegistryBlock, NonBoxedRegBlock, BurnRate}},
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState, network::ids::NetworkID,
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};
pub struct GrassBlock;

impl Block for GrassBlock {
    fn dropped_items(&self, _state: BlockState, _held_item: crate::item::inventory_slot::InventorySlot) -> Vec<ItemStack> {
        vec![ItemStack::new(3, 1, 0)]
    }
    fn id(&self) -> BlockIdentifier {
        2
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
}