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

pub struct MusicDiscItem(pub ItemIdentifier);
impl Item for MusicDiscItem {
    fn id(&self) -> ItemIdentifier {
        self.0
    }

    fn stack_size(&self) -> i8 {
        1
    }

    fn durability(&self) -> Option<i16> {
        None
    }
}