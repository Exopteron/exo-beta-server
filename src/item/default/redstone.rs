use std::ops::Deref;

use hecs::Entity;
use nbt::CompoundTag;
use parking_lot::MutexGuard;

use crate::{
    aabb::{AABB, AABBSize},
    ecs::{entities::player::Chatbox, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{
        item::block::ActionResult,
        stack::{ItemStack, ItemStackType}, inventory_slot::InventorySlot,
    },
    network::ids::NetworkID,
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState, block_entity::{BlockEntityLoader, SignData, BlockEntity, BlockEntitySaver},
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};

use super::door::fd;
pub struct RedstoneItem {}
impl Item for RedstoneItem {
    fn id(&self) -> ItemIdentifier {
        331
    }

    fn stack_size(&self) -> i8 {
        64
    }

    fn durability(&self) -> Option<i16> {
        None
    }
    fn on_use(&self, game: &mut Game, server: &mut crate::server::Server, item: MutexGuard<InventorySlot>, slot: usize, user: Entity, target: Option<crate::item::item::BlockUseTarget>) -> SysResult {
        if let Some(mut target) = target {
            if matches!(target.face, Face::Invalid) || matches!(target.face, Face::NegativeY) {
                return Ok(());
            }
            if !game.is_solid_block(target.position) {
                return Ok(());
            }
            match target.face {
                Face::PositiveY => {
                    target.position.y += 1;
                },
                Face::NegativeZ => {
                    target.position.z -= 1;
                }
                Face::Invalid => unreachable!(),
                Face::NegativeY => (),
                Face::PositiveZ => {
                    target.position.z += 1;
                }
                Face::NegativeX => {
                    target.position.x -= 1;
                }
                Face::PositiveX => {
                    target.position.x += 1;
                }
            }
            if game.is_solid_block(target.position) {
                return Ok(());
            }
            game.set_block(target.position, BlockState::new(55, 0), target.world);
        }
        Ok(())
    }
}

pub struct RedstoneDustBlock;
impl Block for RedstoneDustBlock {
    fn id(&self) -> BlockIdentifier {
        55
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