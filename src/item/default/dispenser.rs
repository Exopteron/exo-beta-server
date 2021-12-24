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

use super::furnace::furnace_orient;

pub struct DispenserBlock;

impl Block for DispenserBlock {
    fn id(&self) -> BlockIdentifier {
        23
    }

    fn item_stack_size(&self) -> i8 {
        64
    }

    fn place(&self, game: &mut Game, entity: Entity, mut item: ItemStack, mut position: BlockPosition, face: Face, world: i32) -> Option<BlockPlacementEvent> {
        position = face.offset(position);
        item.set_damage(furnace_orient(&mut position, &game.ecs.get::<Position>(entity).unwrap()).into());
        Some(BlockPlacementEvent {
            held_item: item,
            location: position,
            face,
            world,
        })
    }
}