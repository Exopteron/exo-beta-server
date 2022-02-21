use hecs::Entity;

use crate::{
    ecs::{entities::player::Chatbox, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{stack::{ItemStackType, ItemStack}, item::block::{ActionResult, AtomicRegistryBlock, NonBoxedRegBlock, BurnRate}},
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState, network::ids::NetworkID, aabb::AABB,
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};

pub struct LeavesBlock;

impl Block for LeavesBlock {
    fn neighbor_update(&self, world: i32, game: &mut Game, position: BlockPosition, state: BlockState, offset: Face, neighbor_state: BlockState) -> SysResult {
        self.tick(world, game, state, position);
        Ok(())
    }
    fn tick(&self, world: i32, game: &mut Game, state: BlockState, position: BlockPosition) {
        let mut keep = false;
        for face in Face::all_faces() {
            let pos = face.offset(position);
            let id = game.block_id_at(pos);
            if id == 18 || id == 17 {
                keep = true;
                break;
            }
        }
        if !keep {
            game.break_block(position, position.world);
        }
    }
    fn burn_rate(&self) -> Option<crate::item::item::block::BurnRate> {
        Some(BurnRate(30, 60))
    }
    fn id(&self) -> BlockIdentifier {
        18
    }

    fn item_stack_size(&self) -> i8 {
        64
    }

    fn dropped_items(&self, _state: BlockState, _held_item: crate::item::inventory_slot::InventorySlot) -> Vec<ItemStack> {
        Vec::new()
    }
    fn is_solid(&self) -> bool {
        false
    }
    fn opaque(&self) -> bool {
        false
    }
}