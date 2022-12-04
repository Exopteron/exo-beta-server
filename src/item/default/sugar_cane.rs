use std::ops::Deref;

use hecs::Entity;
use nbt::CompoundTag;

use crate::{
    aabb::{AABB, AABBSize},
    ecs::{entities::{player::Chatbox, item::ItemEntityBuilder}, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{
        item::block::ActionResult,
        stack::{ItemStack, ItemStackType},
    },
    network::ids::NetworkID,
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState, block_entity::{BlockEntityLoader, SignData, BlockEntity, BlockEntitySaver},
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};

pub struct SugarCane;
impl Block for SugarCane {
    fn dropped_items(&self, state: BlockState, held_item: crate::item::inventory_slot::InventorySlot) -> Vec<ItemStack> {
        vec![ItemStack::new(338, 1, 0)]
    }
    fn hardness(&self) -> i32 {
        1
    }
    fn id(&self) -> BlockIdentifier {
        83
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn can_place_on(&self, world: i32, game: &mut Game, position: BlockPosition, face: Face) -> bool {
        let position = face.offset(position);
        let block_under = game.block_id_at(position.offset(0, -1, 0));
        if block_under == self.id() {
            return true;
        }
        if block_under != 2 && block_under != 3 && block_under != 12 {
            return false;
        }
        let v = game.block_id_at(position.offset(-1, -1, 0));
        if v == 8 || v == 9 {
            return true;
        }
        let v = game.block_id_at(position.offset(1, -1, 0));
        if v == 8 || v == 9 {
            return true;
        }
        let v = game.block_id_at(position.offset(0, -1, -1));
        if v == 8 || v == 9 {
            return true;
        }
        let v = game.block_id_at(position.offset(0, -1, 1));
        if v == 8 || v == 9 {
            return true;
        }
        false
    }
    fn neighbor_update(&self, world: i32, game: &mut Game, position: BlockPosition, state: BlockState, offset: Face, neighbor_state: BlockState) -> SysResult {
        if !self.can_place_on(world, game, position, Face::Invalid) {
            game.break_block(position, world);
            let dropped_item = ItemEntityBuilder::build(game, position.into(), ItemStack::new(338, 1, 0));
            game.spawn_entity(dropped_item);
        }
        Ok(())
    }
    fn is_solid(&self) -> bool {
        false
    }
    fn tick(&self, world: i32, game: &mut Game, mut state: BlockState, position: BlockPosition) {
        if let Some(b) = game.block(position.offset(0, 1, 0), world) {
            if b.is_air() {
                let mut val = 1;
                while game.block(position.offset(0, -val, 0), world).unwrap_or_else(BlockState::air).b_type == 83 {
                    val += 1;
                }
                if val < 3 {
                    if state.b_metadata == 15 {
                        game.set_block(position.offset(0, 1, 0), BlockState::new(83, 0), world);
                        state.b_metadata = 0;
                        game.set_block(position, state, world);
                    } else {
                        game.set_block(position, BlockState::new(83, state.b_metadata + 1), world);
                    }
                }
            }
        }
    }
    fn collision_box(&self, _state: BlockState, _position: BlockPosition) -> Option<AABB> {
        None
    }
    fn opaque(&self) -> bool {
        false
    }
}