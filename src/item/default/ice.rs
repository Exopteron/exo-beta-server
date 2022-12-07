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
pub struct IceBlock;

impl Block for IceBlock {
    fn slipperiness(&self) -> f64 {
        0.98
    }
    fn id(&self) -> BlockIdentifier {
        79
    }
    fn opacity(&self) -> u8 {
        3
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn is_solid(&self) -> bool {
        true
    }
    fn opaque(&self) -> bool {
        false
    }
    fn on_break(&self, game: &mut Game, server: &mut crate::server::Server, breaker: Entity, mut position: BlockPosition, face: Face, world: i32) {
        game.schedule_next_tick(move |game| {
            game.set_block(position, BlockState::new(8, 0), world);
            None
        });
    }

    fn dropped_items(&self, state: BlockState, held_item: crate::item::inventory_slot::InventorySlot) -> Vec<ItemStack> {
        vec![]
    }
}