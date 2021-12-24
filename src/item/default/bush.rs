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

pub struct GenericBush(pub BlockIdentifier);
impl Block for GenericBush {
    fn id(&self) -> BlockIdentifier {
        self.0
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn is_solid(&self) -> bool {
        false
    }
    fn tick(&self, world: i32, game: &mut Game, state: BlockState, position: BlockPosition) {
        if let Err(_) = self.neighbor_update(world, game, position, state, Face::Invalid, state) {

        }
    }
    fn neighbor_update(&self, world: i32, game: &mut Game, position: BlockPosition, state: BlockState, offset: Face, neighbor_state: BlockState) -> SysResult {
        if !self.is_valid_place(world, game, position) {
            game.break_block(position, world);
        }
        Ok(())
    }
    fn can_place_on(&self, world: i32, game: &mut Game, position: BlockPosition, face: Face) -> bool {
        self.is_valid_place(world, game, face.offset(position))
    }
    fn collision_box(&self, state: BlockState, position: BlockPosition) -> Option<crate::aabb::AABB> {
        None
    }
}
impl GenericBush {
    fn is_valid_place(&self, world: i32, game: &mut Game, position: BlockPosition) -> bool {
        let id = game.block_id_at(position.offset(0, -1, 0), world);
        if id == 2 || id == 3 {
            return true;
        }
        false
    }
}