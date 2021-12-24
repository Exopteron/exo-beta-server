use hecs::Entity;

use crate::{
    ecs::{entities::player::Chatbox, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{stack::{ItemStackType, ItemStack}, item::block::{ActionResult, AtomicRegistryBlock, NonBoxedRegBlock}},
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState, network::ids::NetworkID, aabb::AABB,
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};

pub struct SlabBlock;

impl Block for SlabBlock {
    fn id(&self) -> BlockIdentifier {
        44
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn collision_box(&self, _: BlockState, position: BlockPosition) -> Option<AABB> {
        Some(AABB::new(position.x as f64, position.y as f64, position.z as f64, position.x as f64 + 1., position.y as f64 + 0.5, position.z as f64 + 1.))
    }
    fn is_solid(&self) -> bool {
        false
    }
}