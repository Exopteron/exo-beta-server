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
pub struct PumpkinBlock(pub BlockIdentifier);
impl Block for PumpkinBlock {
    fn id(&self) -> BlockIdentifier {
        self.0
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn place(&self, game: &mut Game, entity: Entity, mut item: ItemStack, mut position: BlockPosition, face: Face, world: i32) -> Option<BlockPlacementEvent> {
        position = face.offset(position);
        item.set_damage(pumpkin_orient(&game.ecs.get::<Position>(entity).unwrap()).into());
        Some(BlockPlacementEvent {
            held_item: item,
            location: position,
            face,
            world,
        })
    }
}

pub fn pumpkin_orient(placer_pos: &Position) -> u8 {
    let l = (((((placer_pos.yaw * 4.0) / 360.) as f64) + 2.5).floor() as i32) & 3;
    l as u8
}