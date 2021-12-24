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
pub struct ChestBlock;
impl Block for ChestBlock {
    fn id(&self) -> BlockIdentifier {
        54
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn place(&self, game: &mut Game, entity: Entity, mut item: ItemStack, mut position: BlockPosition, face: Face, world: i32) -> Option<BlockPlacementEvent> {
        None
    }
}

pub fn furnace_orient(pos: &mut BlockPosition, placer_pos: &Position) -> u8 {
    let l = (((((placer_pos.yaw * 4.0) / 360.) as f64) + 0.5).floor() as i32) & 3;
    match l {
        0 => {
            2
        }
        1 => {
            5
        }
        2 => {
            3
        }
        _ => {
            4
        }
    }
}